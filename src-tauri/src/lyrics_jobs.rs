use crate::{
    db::{self, Db, Track},
    lyrics::{self, LyricsQuery, LyricsResult},
    lyrics_ai::{self, AiConfig},
    lyrics_review::{self, ReviewResult},
    lyrics_pipeline,
    ytdlp,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    pub track: LyricsQuery,
    pub state: String,
    pub message: String,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Queue {
    pub running: bool,
    pub jobs: Vec<Job>,
    pub error: String,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Budget {
    day: u64,
    seconds: u32,
}

pub struct LyricsService {
    pub dir: PathBuf,
    pub config: Mutex<AiConfig>,
    pub queue: Mutex<Queue>,
    pub gate: Mutex<()>,
    failures: Mutex<HashMap<String, (std::time::Instant, String)>>,
    pub cancelled: AtomicBool,
    pub model_cancelled: AtomicBool,
    pub downloading: AtomicBool,
    worker: AtomicBool,
    budget: Mutex<Budget>,
}

fn load<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    serde_json::from_slice(&lyrics::read_limited(path, 32 * 1024 * 1024)?)
        .map_err(|_| "Tệp trạng thái lyrics không hợp lệ; bản gốc được giữ nguyên.".into())
}
impl LyricsService {
    pub fn new(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut config: AiConfig = load(&dir.join("settings.json")).unwrap_or_default();
        if config.validate().is_err() {
            config = AiConfig::default();
        }
        let mut queue: Queue = load(&dir.join("queue.json")).unwrap_or_else(|message| Queue {
            error: message,
            ..Default::default()
        });
        queue.running = false;
        for job in &mut queue.jobs {
            if job.state == "running" {
                job.state = "pending".into();
                job.message = "Gián đoạn trước đó; bấm Tiếp tục để thử lại.".into();
            }
        }
        let today = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400;
        let budget = load(&dir.join("budget.json")).unwrap_or(Budget {
            day: today,
            seconds: u32::MAX,
        });
        Ok(Self {
            budget: Mutex::new(budget),
            dir,
            config: Mutex::new(config),
            queue: Mutex::new(queue),
            gate: Mutex::new(()),
            failures: Mutex::new(HashMap::new()),
            cancelled: AtomicBool::new(false),
            model_cancelled: AtomicBool::new(false),
            downloading: AtomicBool::new(false),
            worker: AtomicBool::new(false),
        })
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.dir.join("cache")
    }
    pub fn model_dir(&self) -> PathBuf {
        self.dir.join("models")
    }
    fn reserve_cloud(&self, seconds: u32, config: &AiConfig) -> Result<(), String> {
        let day = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400;
        let mut budget = self.budget.lock().map_err(|_| "Kho ngân sách đang lỗi.")?;
        if budget.day != day {
            *budget = Budget { day, seconds: 0 };
        }
        if seconds == 0 || budget.seconds.saturating_add(seconds) > config.daily_cloud_seconds {
            return Err("AI_BUDGET: Đã hết giới hạn giây âm thanh cloud trong ngày.".into());
        }
        let next = Budget {
            day,
            seconds: budget.seconds + seconds,
        };
        lyrics::atomic_json(&self.dir.join("budget.json"), &next, false)?;
        *budget = next;
        Ok(())
    }
    fn persist_queue(&self, queue: &Queue) -> Result<(), String> {
        if serde_json::to_vec(queue).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
            return Err(
                "Hàng đợi quá lớn để lưu an toàn; hãy giảm dữ liệu thư viện trong lượt này.".into(),
            );
        }
        lyrics::atomic_json(&self.dir.join("queue.json"), queue, false)
    }
}

impl From<Track> for LyricsQuery {
    fn from(t: Track) -> Self {
        Self {
            path: t.path,
            title: t.title,
            artist: t.artist,
            album: t.album,
            duration: t.duration,
            url: t.url,
        }
    }
}
pub(crate) fn valid_query(q: &LyricsQuery) -> Result<(), String> {
    if q.path.is_empty()
        || q.path.len() > 8192
        || q.title.len() > 1024
        || q.artist.len() > 1024
        || q.album.len() > 1024
        || q.url.as_ref().is_some_and(|s| s.len() > 8192)
        || !q.duration.is_finite()
        || q.duration < 0.0
    {
        return Err("Thông tin bài hát không hợp lệ.".into());
    }
    Ok(())
}
fn youtube_url(q: &LyricsQuery) -> Option<String> {
    [q.url.as_deref(), Some(q.path.as_str())]
        .into_iter()
        .flatten()
        .find_map(|s| {
            let u = url::Url::parse(s).ok()?;
            if u.scheme() != "https"
                || ![
                    "www.youtube.com",
                    "youtube.com",
                    "m.youtube.com",
                    "music.youtube.com",
                    "youtu.be",
                ]
                .contains(&u.host_str()?)
            {
                return None;
            }
            ytdlp::extract_youtube_id(s).map(|id| format!("https://www.youtube.com/watch?v={id}"))
        })
}
fn ytdlp_path() -> Result<PathBuf, String> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("yt-dlp.exe"));
        }
    }
    paths.push(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/yt-dlp-x86_64-pc-windows-msvc.exe"),
    );
    paths
        .into_iter()
        .find(|p| p.is_file())
        .ok_or("Chưa có yt-dlp đi kèm ứng dụng.".into())
}
fn captions(q: &LyricsQuery, cancel: &AtomicBool) -> Result<Option<LyricsResult>, String> {
    let Some(url) = youtube_url(q) else {
        return Ok(None);
    };
    if q.duration <= 0.0 {
        return Ok(None);
    }
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let exe = ytdlp_path()?;
    let output = dir
        .path()
        .join("captions.%(ext)s")
        .to_string_lossy()
        .to_string();
    let meta_file = dir.path().join("metadata.json");
    let args: Vec<String> = [
        "--ignore-config",
        "--no-playlist",
        "--skip-download",
        "--dump-single-json",
        "--socket-timeout",
        "10",
        "--retries",
        "0",
        "--",
        &url,
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    if lyrics_ai::run_process(
        &exe,
        &args,
        dir.path(),
        Duration::from_secs(45),
        cancel,
        Some(&meta_file),
    )
    .is_err()
    {
        lyrics_ai::check_cancel(cancel)?;
        return Ok(None);
    }
    let metadata: Value =
        serde_json::from_slice(&lyrics::read_limited(&meta_file, 4 * 1024 * 1024)?)
            .map_err(|_| "Metadata YouTube không hợp lệ.")?;
    if metadata["duration"]
        .as_f64()
        .is_none_or(|d| (d - q.duration).abs() > 3.0)
    {
        return Ok(None);
    }
    for (field, flag, source) in [
        ("subtitles", "--write-subs", "youtube-cc"),
        ("automatic_captions", "--write-auto-subs", "youtube-auto"),
    ] {
        let Some(languages) = original_caption_language(&metadata, field) else {
            continue;
        };
        let args: Vec<String> = [
            "--ignore-config",
            "--no-playlist",
            "--skip-download",
            flag,
            "--sub-langs",
            &languages,
            "--sub-format",
            "json3",
            "--socket-timeout",
            "10",
            "--retries",
            "0",
            "--output",
            &output,
            "--",
            &url,
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        if lyrics_ai::run_process(
            &exe,
            &args,
            dir.path(),
            Duration::from_secs(45),
            cancel,
            None,
        )
        .is_err()
        {
            lyrics_ai::check_cancel(cancel)?;
            continue;
        }
        let mut paths: Vec<_> = fs::read_dir(dir.path())
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|s| s == "json3"))
            .collect();
        paths.sort_by_key(|p| !p.to_string_lossy().contains(".vi."));
        for file in paths {
            let Ok(bytes) = lyrics::read_limited(&file, lyrics::MAX_LYRICS * 4) else {
                continue;
            };
            let Ok(v) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            if let Ok(content) = lyrics::captions_to_lrc(&v, q.duration) {
                return Ok(Some(LyricsResult::new(content, source, true)));
            }
        }
    }
    Ok(None)
}

fn original_caption_language(metadata: &Value, field: &str) -> Option<String> {
    let original = metadata["language"].as_str()?.split('-').next()?;
    if original == "auto" || original.is_empty() {
        return None;
    }
    let tracks = metadata[field].as_object()?;
    for key in [format!("{original}-orig"), original.to_string()] {
        let Some(entries) = tracks.get(&key).and_then(Value::as_array) else {
            continue;
        };
        if entries.iter().any(|entry| {
            entry["ext"].as_str() == Some("json3")
                && entry["url"]
                    .as_str()
                    .and_then(|s| url::Url::parse(s).ok())
                    .is_some_and(|u| !u.query_pairs().any(|(k, _)| k == "tlang"))
        }) {
            return Some(key);
        }
    }
    None
}

fn audio_source(
    q: &LyricsQuery,
    dir: &Path,
    config: &AiConfig,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    if !q.path.starts_with("https://") && !q.path.starts_with("http://") {
        return Ok(PathBuf::from(&q.path));
    }
    if !config.network_lookup {
        return Err("Tra cứu/tải mạng đang tắt. Tải bài về máy trước khi tạo lời.".into());
    }
    let url = youtube_url(q)
        .ok_or("STT cho nguồn stream này chưa hỗ trợ. Tải bài về máy rồi tạo lời.")?;
    if q.duration <= 0.0 || q.duration > config.max_audio_seconds as f64 {
        return Err("Chưa rõ thời lượng hoặc bài quá dài; tải về máy để kiểm tra trước.".into());
    }
    let output = dir.join("source.%(ext)s").to_string_lossy().to_string();
    let args: Vec<String> = [
        "--ignore-config",
        "--no-playlist",
        "--no-part",
        "--format",
        "bestaudio[ext=m4a]/bestaudio[ext=webm]",
        "--max-filesize",
        "64M",
        "--socket-timeout",
        "10",
        "--retries",
        "0",
        "--output",
        &output,
        "--",
        &url,
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    lyrics_ai::run_process(
        &ytdlp_path()?,
        &args,
        dir,
        Duration::from_secs(180),
        cancel,
        None,
    )?;
    fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .is_some_and(|s| s.to_string_lossy().starts_with("source."))
        })
        .ok_or("Không tải được âm thanh để nhận diện.".into())
}

fn pipeline_policy(service: &LyricsService, config: &AiConfig, cancel: &AtomicBool) -> Result<(), String> {
    lyrics_ai::check_cancel(cancel)?;
    let live = service.config.lock().map_err(|_| "Cấu hình lyrics đang lỗi.")?;
    if !live.audio_pipeline || live.gemini_model != config.gemini_model
        || live.provider != config.provider || live.model != config.model || live.language != config.language
        || live.context_rewrite != config.context_rewrite
        || (config.provider != "offline" && !live.cloud_consent)
    { return Err("Quyền hoặc cấu hình AI đã thay đổi; dừng quy trình, chưa xuất lyric.".into()); }
    Ok(())
}

fn pipeline_progress(app: &AppHandle, q: &LyricsQuery, stage: u8, message: &str, state: &str) {
    let service = app.state::<LyricsService>();
    let total = service.config.lock().map(|config| 5 + u8::from(config.context_rewrite)).unwrap_or(5);
    let _ = app.emit("lyrics-progress", serde_json::json!({"path": q.path, "stage": stage, "total": total, "message": message, "state": state}));
    if let Ok(mut queue) = service.queue.lock() {
        if let Some(job) = queue.jobs.iter_mut().find(|job| job.state == "running" && job.track.path == q.path) {
            job.message = message.into();
            let _ = app.emit("lyrics-queue", queue.clone());
        }
    }
}

fn audio_pipeline(app: &AppHandle, q: &LyricsQuery, config: &AiConfig, cancel: &AtomicBool) -> Result<LyricsResult, String> {
    let service = app.state::<LyricsService>();
    let result: Result<LyricsResult, String> = (|| {
        pipeline_progress(app, q, 0, "Chuẩn bị âm thanh cho quy trình nhiều bước", "running");
        pipeline_policy(&service, config, cancel)?;
        let offline_ready = lyrics_ai::models(&service.model_dir()).iter().any(|m| m.name == config.model && m.installed);
        lyrics_pipeline::validate(config, offline_ready)?;
        if lyrics_ai::credential("gemini", None)?.is_none() {
            return Err("AI_AUTH: Chưa lưu khóa Gemini để nghe đối chiếu ngữ cảnh.".into());
        }
        if config.provider != "offline" && lyrics_ai::credential(&config.provider, None)?.is_none() {
            return Err("AI_AUTH: Chưa lưu khóa của AI nhận diện đã chọn.".into());
        }
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let source = audio_source(q, temp.path(), config, cancel)?;
        let audio = lyrics_ai::decode_audio(&source, config.max_audio_seconds, cancel)?;
        let reserve = |seconds| {
            pipeline_policy(&service, config, cancel)?;
            let live = service.config.lock().map_err(|_| "Cấu hình lyrics đang lỗi.")?.clone();
            service.reserve_cloud(seconds, &live)
        };
        let result = lyrics_pipeline::run(&audio, config, cancel, |prompt| {
            pipeline_policy(&service, config, cancel)?;
            if config.provider == "offline" || (prompt.is_some() && config.provider == "gemini") {
                lyrics_ai::offline_with_prompt(&service.model_dir(), &audio, config, cancel, prompt)
            } else {
                reserve(audio.duration().ceil() as u32)?;
                lyrics_ai::cloud_with_prompt(&audio, config, cancel, || pipeline_policy(&service, config, cancel), prompt)
            }
        }, reserve, |stage, message| pipeline_progress(app, q, stage, message, "running"))?;
        pipeline_policy(&service, config, cancel)?;
        Ok(result)
    })();
    if let Err(error) = &result { pipeline_progress(app, q, 0, error, "error"); }
    result
}

pub fn generate(
    app: &AppHandle,
    q: &LyricsQuery,
    ai: bool,
    cancel: &AtomicBool,
    wait: bool,
) -> Result<Option<LyricsResult>, String> {
    valid_query(q)?;
    let service = app.state::<LyricsService>();
    if let Some(item) = lyrics::local(&service.cache_dir(), q) {
        return Ok(Some(item));
    }
    if lyrics::deleted(&service.cache_dir(), q) {
        return Ok(None);
    }
    let _gate = if wait {
        service.gate.lock().map_err(|_| "Bộ tạo lời đang lỗi.")?
    } else {
        service.gate.try_lock().map_err(
            |_| "Đang xử lý lời bài khác. Chờ tác vụ hiện tại hoặc tạm dừng hàng đợi rồi thử lại.",
        )?
    };
    lyrics_ai::check_cancel(cancel)?;
    if let Some(item) = lyrics::local(&service.cache_dir(), q) {
        return Ok(Some(item));
    }
    if lyrics::deleted(&service.cache_dir(), q) {
        return Ok(None);
    }
    if let Ok(old_dir) = app.path().app_cache_dir() {
        if let Some(old) = lyrics::local(&old_dir.join("lyrics"), q) {
            return lyrics::save_generated(&service.cache_dir(), q, &old).map(Some);
        }
    }
    let id = lyrics::key(q);
    if let Some((time, message)) = service
        .failures
        .lock()
        .map_err(|_| "Bộ nhớ lyrics đang lỗi.")?
        .get(&id)
    {
        if time.elapsed() < Duration::from_secs(60) {
            return Err(message.clone());
        }
    }
    let result = generate_uncached(app, q, ai, cancel, false, true);
    if let Err(message) = &result {
        if let Ok(mut failures) = service.failures.lock() {
            failures.retain(|_, (time, _)| time.elapsed() < Duration::from_secs(60));
            if failures.len() < 1024 {
                failures.insert(id, (std::time::Instant::now(), message.clone()));
            }
        }
    }
    result
}
fn generate_uncached(
    app: &AppHandle,
    q: &LyricsQuery,
    ai: bool,
    cancel: &AtomicBool,
    transcribe_only: bool,
    review_after: bool,
) -> Result<Option<LyricsResult>, String> {
    let service = app.state::<LyricsService>();
    let config = service
        .config
        .lock()
        .map_err(|_| "Cấu hình lyrics đang lỗi.")?
        .clone();
    let original_key = lyrics::key(q);
    let full_pipeline = config.audio_pipeline && ai && review_after && config.provider == "gemini";
    let mut result = if full_pipeline {
        match audio_pipeline(app, q, &config, cancel) {
            Ok(item) => Some(item),
            Err(_) if config.provider != "gemini" => {
                None
            }
            Err(e) => return Err(e),
        }
    } else {
        None
    };
    if result.is_none() && config.network_lookup && !transcribe_only {
        result = lyrics::fetch_online(q, config.lyricsplus_lookup, &config.language)
            .ok()
            .flatten();
        lyrics_ai::check_cancel(cancel)?;
        if result.is_none() {
            result = captions(q, cancel).ok().flatten();
            lyrics_ai::check_cancel(cancel)?;
        }
    }
    if result.is_none() && ai {
        let offline_ready = lyrics_ai::models(&service.model_dir())
            .into_iter()
            .any(|m| m.name == config.model && m.installed);
        let cloud_ready = config.provider != "offline" && config.cloud_consent;
        if !offline_ready && !cloud_ready {
            return Err("Chưa có model offline hoặc quyền cloud. Mở Cài đặt → Lời bài hát.".into());
        }
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let source = audio_source(q, temp.path(), &config, cancel)?;
        let audio = lyrics_ai::decode_audio(&source, config.max_audio_seconds, cancel)?;
        if cloud_ready {
            if lyrics_ai::credential(&config.provider, None)?.is_none() {
                return Err("AI_AUTH: Chưa lưu API key trong cài đặt.".into());
            }
            let policy = || {
                let live = service
                    .config
                    .lock()
                    .map_err(|_| "Cấu hình lyrics đang lỗi.")?;
                if !live.cloud_consent || live.provider != config.provider {
                    return Err("Quyền cloud đã thay đổi; không gửi âm thanh.".into());
                }
                Ok(())
            };
            policy()?;
            let live_budget = service
                .config
                .lock()
                .map_err(|_| "Cấu hình đang lỗi.")?
                .clone();
            service.reserve_cloud(audio.duration().ceil() as u32, &live_budget)?;
            match lyrics_ai::cloud(&audio, &config, cancel, policy) {
                Ok(item) => result = Some(item),
                Err(e) if e.starts_with("AI_QUOTA") || e.starts_with("AI_AUTH") => return Err(e),
                Err(e) => {
                    if config.offline_fallback && offline_ready {
                        result = Some(lyrics_ai::offline(
                            &service.model_dir(),
                            &audio,
                            &config,
                            cancel,
                        )?);
                    } else {
                        return Err(e);
                    }
                }
            }
        } else {
            result = Some(lyrics_ai::offline(
                &service.model_dir(),
                &audio,
                &config,
                cancel,
            )?);
        }
    }
    if let Some(mut item) = result {
        if review_after && config.auto_review && !full_pipeline {
            match review_with_policy(&service, &item.content, &config, cancel) {
                Ok(reviewed) => {
                    item.content = reviewed.content;
                    item.notice = Some(format!(
                        "AI đã rà soát chính tả và dấu câu: {} dòng thay đổi. Chưa xác minh khớp giọng hát; hãy nghe kiểm tra.",
                        reviewed.changed_lines
                    ));
                }
                Err(e) => item.notice = Some(format!("Đã có lời nhưng chưa rà soát AI: {e}")),
            }
            item.review = true;
        }
        if original_key != lyrics::key(q) {
            let error = "Tệp nhạc đã thay đổi trong lúc tạo lời; không lưu kết quả cũ.";
            if full_pipeline { pipeline_progress(app, q, 0, error, "error"); }
            return Err(error.into());
        }
        if full_pipeline { pipeline_policy(&service, &config, cancel)?; }
        if transcribe_only {
            if full_pipeline {
                if item.draft_only { pipeline_progress(app, q, 0, "Còn chỗ chưa nghe rõ · chỉ trả bản nháp có đánh dấu", "needs_review"); }
                else { pipeline_progress(app, q, 5 + u8::from(config.context_rewrite), "Đã hoàn thành lyric nhiều bước · xem trước trước khi lưu", "done"); }
            }
            return Ok(Some(item));
        }
        let saved = lyrics::save_generated(&service.cache_dir(), q, &item)?;
        let _ = app.emit("lyrics-updated", &q.path);
        if full_pipeline {
            if saved.draft_only { pipeline_progress(app, q, 0, "Đã giữ bản nháp chưa rõ · không tự gửi AI lại", "needs_review"); }
            else { pipeline_progress(app, q, 5 + u8::from(config.context_rewrite), "Đã hoàn thành và lưu lyric nhiều bước", "done"); }
        }
        Ok(Some(saved))
    } else {
        lyrics_ai::check_cancel(cancel)?;
        Ok(None)
    }
}

fn review_with_policy(
    service: &LyricsService,
    content: &str,
    config: &AiConfig,
    cancel: &AtomicBool,
) -> Result<ReviewResult, String> {
    lyrics_review::review(content, config, || {
        lyrics_ai::check_cancel(cancel)?;
        let live = service
            .config
            .lock()
            .map_err(|_| "Cấu hình lyrics đang lỗi.")?;
        if !live.text_consent || live.gemini_model != config.gemini_model {
            return Err("Quyền rà soát hoặc model đã thay đổi. Không gửi lời.".into());
        }
        Ok(())
    })
}

#[tauri::command]
pub async fn lyrics_review_text(app: AppHandle, content: String) -> Result<ReviewResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<LyricsService>();
        let _gate = service
            .gate
            .try_lock()
            .map_err(|_| "Đang có tác vụ lời khác. Chờ xong hoặc tạm dừng hàng đợi.")?;
        let config = service
            .config
            .lock()
            .map_err(|_| "Cấu hình lyrics đang lỗi.")?
            .clone();
        review_with_policy(&service, &content, &config, &AtomicBool::new(false))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn lyrics_transcribe(app: AppHandle, track: LyricsQuery) -> Result<LyricsResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        valid_query(&track)?;
        let service = app.state::<LyricsService>();
        let _gate = service
            .gate
            .try_lock()
            .map_err(|_| "Đang có tác vụ lời khác. Chờ xong hoặc tạm dừng hàng đợi.")?;
        generate_uncached(&app, &track, true, &AtomicBool::new(false), true, false)?
            .ok_or_else(|| "Chưa nhận diện được lời từ âm thanh.".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
pub struct SettingsView {
    pub config: AiConfig,
    pub keys: Vec<String>,
    pub models: Vec<lyrics_ai::ModelStatus>,
    pub downloading: bool,
    pub cloud_seconds_used: u32,
    pub lyrics_dir: String,
    pub lyrics_count: usize,
}
#[tauri::command]
pub fn open_lyrics_folder(app: AppHandle) -> Result<String, String> {
    let s = app.state::<LyricsService>();
    let dir = s.cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }
    Ok(dir.to_string_lossy().into_owned())
}
#[tauri::command]
pub async fn lyrics_settings(app: AppHandle) -> Result<SettingsView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.state::<LyricsService>();
        let keys = ["groq", "gemini", "openai"]
            .into_iter()
            .filter(|p| lyrics_ai::credential(p, None).ok().flatten().is_some())
            .map(str::to_string)
            .collect();
        let config = s.config.lock().map_err(|_| "Cấu hình đang lỗi.")?.clone();
        let budget = s.budget.lock().map_err(|_| "Ngân sách đang lỗi.")?;
        let today = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400;
        let cache_dir = s.cache_dir();
        let lyrics_count = std::fs::read_dir(&cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext == "json" || ext == "lrc")
                    })
                    .count()
            })
            .unwrap_or(0);
        Ok(SettingsView {
            config,
            keys,
            models: lyrics_ai::models(&s.model_dir()),
            downloading: s.downloading.load(Ordering::Relaxed),
            cloud_seconds_used: if today == budget.day {
                budget.seconds
            } else {
                0
            },
            lyrics_dir: cache_dir.to_string_lossy().into_owned(),
            lyrics_count,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn lyrics_save_settings(
    app: AppHandle,
    config: AiConfig,
    api_key: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        config.validate()?;
        let s = app.state::<LyricsService>();
        if let Some(key) = api_key {
            lyrics_ai::credential(&config.provider, Some(&key))?;
        }
        let mut stored = s.config.lock().map_err(|_| "Cấu hình đang lỗi.")?;
        lyrics::atomic_json(&s.dir.join("settings.json"), &config, false)?;
        *stored = config;
        service_clear_failures(&s);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
fn service_clear_failures(s: &LyricsService) {
    if let Ok(mut failures) = s.failures.lock() {
        failures.clear();
    }
}
#[tauri::command]
pub async fn lyrics_store_key(provider: String, api_key: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        lyrics_ai::credential(&provider, Some(&api_key)).map(|_| ())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn lyrics_read(
    app: AppHandle,
    track: LyricsQuery,
) -> Result<Option<LyricsResult>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let automatic = app
            .state::<LyricsService>()
            .config
            .lock()
            .map_err(|_| "Cấu hình đang lỗi.")?
            .automatic;
        generate(&app, &track, automatic, &AtomicBool::new(false), false)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn lyrics_download_model(app: AppHandle, model: String) -> Result<(), String> {
    let s = app.state::<LyricsService>();
    if s.downloading.swap(true, Ordering::SeqCst) {
        return Err("Đang có lượt tải model.".into());
    }
    s.model_cancelled.store(false, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.state::<LyricsService>();
        let last = std::cell::Cell::new(std::time::Instant::now());
        let result =
            lyrics_ai::install_model(&s.model_dir(), &model, &s.model_cancelled, |done, total| {
                if last.get().elapsed() > Duration::from_millis(250) || done == total {
                    let _ = app.emit(
                        "lyrics-model-progress",
                        serde_json::json!({"done":done,"total":total}),
                    );
                    last.set(std::time::Instant::now());
                }
            });
        s.downloading.store(false, Ordering::SeqCst);
        if result.is_ok() {
            service_clear_failures(&s);
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn lyrics_cancel_download(app: AppHandle) {
    app.state::<LyricsService>()
        .model_cancelled
        .store(true, Ordering::SeqCst);
}
#[tauri::command]
pub async fn lyrics_delete(app: AppHandle, track: LyricsQuery) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        valid_query(&track)?;
        let service = app.state::<LyricsService>();
        let _gate = service
            .gate
            .try_lock()
            .map_err(|_| "Đang có tác vụ lời khác. Chờ xong hoặc tạm dừng hàng đợi rồi xóa.")?;
        lyrics::delete_saved(&service.cache_dir(), &track)?;
        service_clear_failures(&service);
        let _ = app.emit("lyrics-updated", &track.path);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn lyrics_generate(
    app: AppHandle,
    track: LyricsQuery,
    refresh_online: Option<bool>,
    rewrite: Option<bool>,
) -> Result<LyricsResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        valid_query(&track)?;
        let service = app.state::<LyricsService>();
        if rewrite == Some(true)
            || (refresh_online != Some(true) && lyrics::deleted(&service.cache_dir(), &track))
        {
            let _gate = service
                .gate
                .try_lock()
                .map_err(|_| "Đang có tác vụ lời khác. Chờ xong hoặc tạm dừng hàng đợi.")?;
            return generate_uncached(&app, &track, true, &AtomicBool::new(false), true, true)?
                .ok_or_else(|| "Chưa nhận diện được lời mới. Bản cũ được giữ nguyên.".into());
        }
        if refresh_online == Some(true) {
            valid_query(&track)?;
            let service = app.state::<LyricsService>();
            let config = service
                .config
                .lock()
                .map_err(|_| "Cấu hình lyrics đang lỗi.")?
                .clone();
            if !config.network_lookup {
                return Err("Tra cứu mạng đang tắt. Bật trong Cài đặt → Lời bài hát.".into());
            }
            // Preview only: never upload audio, overwrite a cache, or replace a manual edit here.
            return lyrics::fetch_online(&track, config.lyricsplus_lookup, &config.language)?
                .ok_or_else(|| {
                    "Chưa tìm thấy lời có sẵn khớp bài. Bản đã lưu được giữ nguyên.".into()
                });
        }
        generate(&app, &track, true, &AtomicBool::new(false), false)?
            .ok_or("Chưa tìm thấy lời bài hát.".into())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn lyrics_queue(app: AppHandle) -> Result<Queue, String> {
    app.state::<LyricsService>()
        .queue
        .lock()
        .map(|q| q.clone())
        .map_err(|_| "Hàng đợi đang lỗi.".into())
}
#[tauri::command]
pub fn lyrics_pause(app: AppHandle) -> Result<(), String> {
    let s = app.state::<LyricsService>();
    s.cancelled.store(true, Ordering::SeqCst);
    let mut q = s.queue.lock().map_err(|_| "Hàng đợi đang lỗi.")?;
    q.running = false;
    s.persist_queue(&q)?;
    let _ = app.emit("lyrics-queue", q.clone());
    Ok(())
}
#[tauri::command]
pub async fn lyrics_start_library(app: AppHandle, resume: bool) -> Result<Queue, String> {
    let s = app.state::<LyricsService>();
    if s.worker.swap(true, Ordering::SeqCst) {
        return Err("Hàng đợi đang chạy hoặc đang kết thúc tác vụ hiện tại.".into());
    }
    let setup = (|| {
        let tracks = if resume {
            Vec::new()
        } else {
            db::list_tracks(
                &*app
                    .state::<Db>()
                    .0
                    .lock()
                    .map_err(|_| "Thư viện đang bận.")?,
                "",
            )
            .map_err(|e| e.to_string())?
        };
        let mut guard = s.queue.lock().map_err(|_| "Hàng đợi đang lỗi.")?;
        let mut q = guard.clone();
        if !resume {
            if q.jobs
                .iter()
                .any(|j| j.state == "pending" || j.state == "running")
            {
                return Err("Còn hàng đợi chưa xong. Bấm Tiếp tục trước khi tạo lượt mới.".into());
            }
            let mut seen = HashSet::new();
            q.jobs = tracks
                .into_iter()
                .map(LyricsQuery::from)
                .filter(|t| seen.insert(lyrics::key(t)))
                .map(|track| Job {
                    track,
                    state: "pending".into(),
                    message: String::new(),
                })
                .collect();
        }
        q.running = true;
        q.error.clear();
        s.persist_queue(&q)?;
        s.cancelled.store(false, Ordering::SeqCst);
        *guard = q.clone();
        Ok(q)
    })();
    let snapshot = match setup {
        Ok(q) => q,
        Err(e) => {
            s.worker.store(false, Ordering::SeqCst);
            return Err(e);
        }
    };
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || run_queue(&handle));
    Ok(snapshot)
}
fn run_queue(app: &AppHandle) {
    let s = app.state::<LyricsService>();
    let limit = s.config.lock().map(|c| c.batch_limit).unwrap_or(1);
    let mut processed = 0;
    loop {
        if s.cancelled.load(Ordering::Relaxed) || processed >= limit {
            break;
        }
        let next = {
            let Ok(mut q) = s.queue.lock() else {
                break;
            };
            let Some(i) = q.jobs.iter().position(|j| j.state == "pending") else {
                break;
            };
            q.jobs[i].state = "running".into();
            q.jobs[i].message = "Đang tìm hoặc nhận diện lời…".into();
            if let Err(e) = s.persist_queue(&q) {
                q.jobs[i].state = "pending".into();
                q.error = e;
                break;
            }
            let _ = app.emit("lyrics-queue", q.clone());
            (i, q.jobs[i].track.clone())
        };
        let existing = lyrics::local(&s.cache_dir(), &next.1).is_some();
        let result = generate(app, &next.1, true, &s.cancelled, true);
        processed += 1;
        let Ok(mut q) = s.queue.lock() else {
            break;
        };
        let job = &mut q.jobs[next.0];
        let mut halt = false;
        if s.cancelled.load(Ordering::Relaxed) && result.is_err() {
            job.state = "pending".into();
            job.message = "Đã tạm dừng; tác vụ cloud đang gửi không thể thu hồi.".into();
        } else {
            match result {
                Ok(Some(item)) => {
                    if item.draft_only {
                        job.state = "needs_review".into();
                        job.message = "Bản nháp có chỗ chưa nghe rõ; mở lời để nghe và sửa, không tự gửi lại AI.".into();
                    } else {
                    job.state = if existing { "skipped" } else { "done" }.into();
                    job.message = format!(
                        "{}{}",
                        item.source,
                        if item.review {
                            " · cần kiểm tra"
                        } else {
                            ""
                        }
                    );
                    }
                }
                Ok(None) => {
                    job.state = "failed".into();
                    job.message = "Không tìm thấy lời.".into();
                }
                Err(e) => {
                    halt = e.starts_with("AI_QUOTA")
                        || e.starts_with("AI_AUTH")
                        || e.starts_with("AI_BUDGET");
                    job.state = if halt { "pending" } else { "failed" }.into();
                    job.message = e;
                }
            }
        }
        if halt {
            q.running = false;
        }
        if let Err(e) = s.persist_queue(&q) {
            q.error = e;
            break;
        }
        let _ = app.emit("lyrics-queue", q.clone());
        if halt {
            break;
        }
        drop(q);
        for _ in 0..10 {
            if s.cancelled.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    if let Ok(mut q) = s.queue.lock() {
        q.running = false;
        if let Err(e) = s.persist_queue(&q) {
            q.error = e;
        }
        s.worker.store(false, Ordering::SeqCst);
        let _ = app.emit("lyrics-queue", q.clone());
    } else {
        s.worker.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_queue_is_paused_and_budget_is_durable() {
        let dir = tempfile::tempdir().unwrap();
        let t = LyricsQuery {
            path: "a.mp3".into(),
            title: "A".into(),
            artist: "B".into(),
            album: "".into(),
            duration: 30.0,
            url: None,
        };
        lyrics::atomic_json(
            &dir.path().join("queue.json"),
            &Queue {
                running: true,
                error: String::new(),
                jobs: vec![Job {
                    track: t,
                    state: "running".into(),
                    message: "".into(),
                }],
            },
            false,
        )
        .unwrap();
        let s = LyricsService::new(dir.path().into()).unwrap();
        assert!(!s.queue.lock().unwrap().running);
        assert_eq!(s.queue.lock().unwrap().jobs[0].state, "pending");
        let c = AiConfig {
            daily_cloud_seconds: 60,
            ..Default::default()
        };
        s.reserve_cloud(40, &c).unwrap();
        assert!(s.reserve_cloud(30, &c).is_err());
        let reopened = LyricsService::new(dir.path().into()).unwrap();
        assert!(reopened.reserve_cloud(30, &c).is_err());
    }
    #[test]
    fn pipeline_rechecks_consent_configuration_and_each_cloud_pass_budget() {
        let dir = tempfile::tempdir().unwrap();
        let service = LyricsService::new(dir.path().into()).unwrap();
        let config = AiConfig { audio_pipeline: true, provider: "groq".into(), cloud_consent: true, daily_cloud_seconds: 6, ..Default::default() };
        *service.config.lock().unwrap() = config.clone();
        let cancel = AtomicBool::new(false);
        assert!(pipeline_policy(&service, &config, &cancel).is_ok());
        for _ in 0..3 { service.reserve_cloud(2, &config).unwrap(); }
        assert!(service.reserve_cloud(1, &config).unwrap_err().starts_with("AI_BUDGET"));
        service.config.lock().unwrap().audio_pipeline = false;
        assert!(pipeline_policy(&service, &config, &cancel).is_err());
        *service.config.lock().unwrap() = config.clone();
        service.config.lock().unwrap().cloud_consent = false;
        assert!(pipeline_policy(&service, &config, &cancel).is_err());
        *service.config.lock().unwrap() = config.clone();
        service.config.lock().unwrap().provider = "openai".into();
        assert!(pipeline_policy(&service, &config, &cancel).is_err());
        *service.config.lock().unwrap() = config.clone();
        service.config.lock().unwrap().context_rewrite = true;
        assert!(pipeline_policy(&service, &config, &cancel).is_err());
        *service.config.lock().unwrap() = config.clone();
        cancel.store(true, Ordering::SeqCst);
        assert!(pipeline_policy(&service, &config, &cancel).is_err());
    }
    #[test]
    fn corrupt_budget_cannot_reset_spending_limit() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("budget.json"), b"broken").unwrap();
        let s = LyricsService::new(dir.path().into()).unwrap();
        assert!(s.reserve_cloud(1, &AiConfig::default()).is_err());
        assert_eq!(fs::read(dir.path().join("budget.json")).unwrap(), b"broken");
    }
    #[test]
    fn translated_captions_are_not_used_as_original_lyrics() {
        let metadata = serde_json::json!({"language":"en","subtitles":{"en":[{"ext":"json3","url":"https://www.youtube.com/api/timedtext?lang=en"}],"vi":[{"ext":"json3","url":"https://www.youtube.com/api/timedtext?tlang=vi"}]}});
        assert_eq!(
            original_caption_language(&metadata, "subtitles"),
            Some("en".into())
        );
        let mut unknown_language = metadata.clone();
        unknown_language.as_object_mut().unwrap().remove("language");
        assert_eq!(
            original_caption_language(&unknown_language, "subtitles"),
            None
        );
        let translated = serde_json::json!({"language":"vi","automatic_captions":{"vi":[{"ext":"json3","url":"https://www.youtube.com/api/timedtext?tlang=vi"}]}});
        assert_eq!(
            original_caption_language(&translated, "automatic_captions"),
            None
        );
    }
    #[test]
    fn fake_youtube_hosts_are_rejected() {
        let mut q = LyricsQuery {
            path: "https://youtube.com.attacker.test/watch?v=12345678901".into(),
            title: "".into(),
            artist: "".into(),
            album: "".into(),
            duration: 30.0,
            url: None,
        };
        assert!(youtube_url(&q).is_none());
        q.path = "https://youtu.be/12345678901".into();
        assert!(youtube_url(&q).is_some());
    }
}
