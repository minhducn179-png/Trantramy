use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::{Command as ShellCommand, CommandEvent};
use tauri_plugin_shell::ShellExt;

fn http_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(3))
            .timeout_read(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .build()
    })
}

pub fn extract_youtube_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_lowercase();
    if !lower.contains("youtube.com") && !lower.contains("youtu.be") {
        return None;
    }
    if lower.contains("/playlist") && !lower.contains("v=") {
        return None;
    }
    let candidate = if let Some(idx) = trimmed.find("v=") {
        let rest = &trimmed[idx + 2..];
        rest.chars()
            .take_while(|c| *c != '&' && *c != '#' && *c != '?' && *c != '/')
            .collect::<String>()
    } else if let Some(idx) = lower.find("youtu.be/") {
        let rest = &trimmed[idx + 9..];
        rest.chars()
            .take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/')
            .collect::<String>()
    } else if let Some(idx) = lower.find("/shorts/") {
        let rest = &trimmed[idx + 8..];
        rest.chars()
            .take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/')
            .collect::<String>()
    } else if let Some(idx) = lower.find("/embed/") {
        let rest = &trimmed[idx + 7..];
        rest.chars()
            .take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/')
            .collect::<String>()
    } else {
        String::new()
    };

    if candidate.len() == 11
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(candidate)
    } else {
        None
    }
}

pub fn canonical_url(url: &str) -> String {
    if let Some(vid) = extract_youtube_id(url) {
        return format!("https://www.youtube.com/watch?v={vid}");
    }
    url.trim().to_string()
}

fn url_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn parse_duration_text(s: &str) -> f64 {
    let parts: Vec<f64> = s
        .trim()
        .split(':')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect();
    match parts.as_slice() {
        [m, sec] => m * 60.0 + sec,
        [h, m, sec] => h * 3600.0 + m * 60.0 + sec,
        [sec] => *sec,
        _ => 0.0,
    }
}

fn fast_youtube_video_info(url: &str) -> Option<OnlineTrack> {
    let vid = extract_youtube_id(url)?;
    let canon = format!("https://www.youtube.com/watch?v={vid}");
    let oembed_url = format!(
        "https://www.youtube.com/oembed?url={}&format=json",
        url_encode_query(&canon)
    );
    let resp = http_agent().get(&oembed_url).call().ok()?;
    let val: Value = resp.into_json().ok()?;
    let title = val.get("title")?.as_str()?.trim().to_string();
    if title.is_empty() {
        return None;
    }
    let artist = val
        .get("author_name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("YouTube")
        .to_string();
    let thumbnail = Some(format!("https://i.ytimg.com/vi/{vid}/hqdefault.jpg"));
    Some(OnlineTrack {
        title,
        artist,
        album: "YouTube".to_string(),
        duration: 0.0,
        thumbnail,
        url: canon,
        source: "youtube".to_string(),
        reason: None,
    })
}

fn fast_soundcloud_track_info(url: &str) -> Option<OnlineTrack> {
    let clean = url.trim().split('?').next().unwrap_or(url.trim());
    let lower = clean.to_lowercase();
    if !lower.contains("soundcloud.com/")
        || lower.contains("/sets/")
        || lower.ends_with("/tracks")
        || lower.ends_with("/likes")
        || lower.ends_with("/Reposts")
    {
        return None;
    }
    // Check path segments: soundcloud.com/<user>/<track>
    let after_host = clean.split("soundcloud.com/").nth(1)?;
    let segs: Vec<&str> = after_host.trim_matches('/').split('/').collect();
    if segs.len() != 2 || segs[0].is_empty() || segs[1].is_empty() {
        return None;
    }
    let oembed_url = format!(
        "https://soundcloud.com/oembed?url={}&format=json",
        url_encode_query(clean)
    );
    let resp = http_agent().get(&oembed_url).call().ok()?;
    let val: Value = resp.into_json().ok()?;
    let raw_title = val.get("title")?.as_str()?.trim();
    if raw_title.is_empty() {
        return None;
    }
    let artist = val
        .get("author_name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("SoundCloud")
        .to_string();
    let suffix = format!(" by {artist}");
    let title = raw_title
        .strip_suffix(&suffix)
        .unwrap_or(raw_title)
        .trim()
        .to_string();
    let thumbnail = val
        .get("thumbnail_url")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    Some(OnlineTrack {
        title,
        artist,
        album: "SoundCloud".to_string(),
        duration: 0.0,
        thumbnail,
        url: clean.to_string(),
        source: "soundcloud".to_string(),
        reason: None,
    })
}

fn collect_yt_video_renderers(v: &Value, limit: usize, out: &mut Vec<OnlineTrack>) {
    if out.len() >= limit {
        return;
    }
    match v {
        Value::Object(map) => {
            if let Some(vr) = map.get("videoRenderer") {
                if let Some(track) = parse_video_renderer(vr) {
                    if !out.iter().any(|x| x.url == track.url) {
                        out.push(track);
                        if out.len() >= limit {
                            return;
                        }
                    }
                }
            }
            for val in map.values() {
                collect_yt_video_renderers(val, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_yt_video_renderers(item, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn parse_video_renderer(vr: &Value) -> Option<OnlineTrack> {
    let vid = vr.get("videoId")?.as_str()?.trim();
    if vid.len() != 11 {
        return None;
    }
    let title = vr
        .pointer("/title/runs/0/text")
        .and_then(|x| x.as_str())
        .or_else(|| vr.pointer("/title/simpleText").and_then(|x| x.as_str()))?
        .trim()
        .to_string();
    if title.is_empty() {
        return None;
    }
    let artist = vr
        .pointer("/ownerText/runs/0/text")
        .and_then(|x| x.as_str())
        .or_else(|| vr.pointer("/longBylineText/runs/0/text").and_then(|x| x.as_str()))
        .or_else(|| vr.pointer("/shortBylineText/runs/0/text").and_then(|x| x.as_str()))
        .unwrap_or("YouTube")
        .trim()
        .to_string();
    let duration = vr
        .pointer("/lengthText/simpleText")
        .and_then(|x| x.as_str())
        .map(parse_duration_text)
        .unwrap_or(0.0);
    Some(OnlineTrack {
        title,
        artist,
        album: "YouTube".to_string(),
        duration,
        thumbnail: Some(format!("https://i.ytimg.com/vi/{vid}/hqdefault.jpg")),
        url: format!("https://www.youtube.com/watch?v={vid}"),
        source: "youtube".to_string(),
        reason: None,
    })
}

fn fast_youtube_search(query: &str, limit: usize) -> Option<Vec<OnlineTrack>> {
    let url = format!(
        "https://www.youtube.com/results?search_query={}&hl=vi",
        url_encode_query(query)
    );
    let resp = http_agent()
        .get(&url)
        .set("Accept-Language", "vi-VN,vi;q=0.9,en-US;q=0.8,en;q=0.7")
        .call()
        .ok()?;
    let html = resp.into_string().ok()?;
    let markers = ["var ytInitialData = ", "window[\"ytInitialData\"] = ", "ytInitialData = "];
    let mut json_slice: Option<&str> = None;
    for m in markers {
        if let Some(pos) = html.find(m) {
            let after = &html[pos + m.len()..];
            if let Some(end) = after.find(";</script>") {
                json_slice = Some(after[..end].trim());
                break;
            }
        }
    }
    let raw_json = json_slice?;
    let val: Value = serde_json::from_str(raw_json).ok()?;
    let mut out = Vec::with_capacity(limit);
    collect_yt_video_renderers(&val, limit, &mut out);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OnlineTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub thumbnail: Option<String>,
    pub url: String,
    pub source: String,
    pub reason: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct DownloadProgress {
    pub url: String,
    pub track_id: Option<i64>,
    pub percent: f64,
    pub status: String,
}

pub struct StreamCache {
    pub map: Mutex<HashMap<String, (String, Instant)>>,
    pub inflight: Mutex<HashSet<String>>,
}

impl StreamCache {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashSet::new()),
        }
    }

    pub fn get(&self, url: &str) -> Option<String> {
        let canon = canonical_url(url);
        let mut map = self.map.lock().unwrap();
        for key in [canon.as_str(), url] {
            if let Some((stream, ts)) = map.get(key) {
                if ts.elapsed() < Duration::from_secs(5 * 3600) {
                    return Some(stream.clone());
                }
                map.remove(key);
            }
        }
        None
    }

    pub fn put(&self, url: String, stream: String) {
        let canon = canonical_url(&url);
        let mut map = self.map.lock().unwrap();
        let now = Instant::now();
        map.insert(canon, (stream.clone(), now));
        map.insert(url, (stream, now));
    }

    pub fn begin_inflight(&self, key: &str) -> bool {
        self.inflight.lock().unwrap().insert(key.to_string())
    }

    pub fn end_inflight(&self, key: &str) {
        self.inflight.lock().unwrap().remove(key);
    }

    pub fn is_inflight(&self, key: &str) -> bool {
        self.inflight.lock().unwrap().contains(key)
    }
}

struct InflightGuard<'a> {
    cache: &'a StreamCache,
    key: String,
}

impl<'a> Drop for InflightGuard<'a> {
    fn drop(&mut self) {
        self.cache.end_inflight(&self.key);
    }
}

pub struct SearchCache(pub Mutex<HashMap<String, (Vec<OnlineTrack>, Instant)>>);

impl SearchCache {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, key: &str) -> Option<Vec<OnlineTrack>> {
        let mut map = self.0.lock().unwrap();
        if let Some((tracks, ts)) = map.get(key) {
            if ts.elapsed() < Duration::from_secs(2 * 3600) && !tracks.is_empty() {
                return Some(tracks.clone());
            }
            map.remove(key);
        }
        None
    }

    pub fn put(&self, key: String, tracks: Vec<OnlineTrack>) {
        if !tracks.is_empty() {
            self.0.lock().unwrap().insert(key, (tracks, Instant::now()));
        }
    }
}

fn str_field(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() && trimmed != "NA" && trimmed != "none" {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn infer_source(v: &Value, url: &str, default_source: &str) -> String {
    let ext = str_field(v, &["extractor_key", "extractor", "ie_key"])
        .unwrap_or_default()
        .to_lowercase();
    let u = url.to_lowercase();
    if ext.contains("youtube") || u.contains("youtube.com") || u.contains("youtu.be") {
        "youtube".to_string()
    } else if ext.contains("soundcloud") || u.contains("soundcloud.com") {
        "soundcloud".to_string()
    } else {
        default_source.to_string()
    }
}

fn extract_thumbnail(v: &Value, id: Option<&str>, source: &str) -> Option<String> {
    if let Some(t) = str_field(v, &["thumbnail"]) {
        if t.starts_with("http") {
            return Some(t);
        }
    }
    if let Some(arr) = v.get("thumbnails").and_then(|x| x.as_array()) {
        for item in arr.iter().rev() {
            let tid = item.get("id").and_then(|x| x.as_str()).unwrap_or("");
            if tid == "original" && arr.len() > 1 {
                continue;
            }
            if let Some(u) = item.get("url").and_then(|x| x.as_str()) {
                if u.starts_with("http") {
                    return Some(u.to_string());
                }
            }
        }
    }
    if source == "youtube" {
        if let Some(vid) = id {
            if !vid.is_empty() {
                return Some(format!("https://i.ytimg.com/vi/{vid}/hqdefault.jpg"));
            }
        }
    }
    None
}

fn parse_entry(v: &Value, default_source: &str) -> Option<OnlineTrack> {
    let id = str_field(v, &["id"]);
    let raw_url = str_field(v, &["webpage_url", "original_url", "url"]);
    let mut source = infer_source(v, raw_url.as_deref().unwrap_or(""), default_source);

    let url = match raw_url {
        Some(u) if u.starts_with("http://") || u.starts_with("https://") => u,
        _ => {
            let vid = id.as_deref()?;
            if source == "youtube" || default_source == "youtube" {
                source = "youtube".to_string();
                format!("https://www.youtube.com/watch?v={vid}")
            } else {
                return None;
            }
        }
    };

    let title = str_field(v, &["track", "title"])?;
    // Skip deleted/private placeholders in playlists
    if title == "[Deleted video]" || title == "[Private video]" {
        return None;
    }

    let artist = str_field(v, &["artist", "creator", "uploader", "channel"])
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = str_field(v, &["album"]).unwrap_or_else(|| {
        if source == "soundcloud" {
            "SoundCloud".to_string()
        } else {
            "YouTube".to_string()
        }
    });
    let duration = v
        .get("duration")
        .and_then(|d| d.as_f64())
        .unwrap_or(0.0);
    let thumbnail = extract_thumbnail(v, id.as_deref(), &source).or_else(|| {
        if source == "youtube" || url.contains("youtube.com") || url.contains("youtu.be") {
            let vid = if let Some(idx) = url.find("v=") {
                let rest = &url[idx + 2..];
                let s: String = rest.chars().take_while(|c| *c != '&' && *c != '#').collect();
                if s.len() == 11 { Some(s) } else { None }
            } else if let Some(idx) = url.find("youtu.be/") {
                let rest = &url[idx + 9..];
                let s: String = rest.chars().take_while(|c| *c != '?' && *c != '&').collect();
                if s.len() == 11 { Some(s) } else { None }
            } else {
                None
            }?;
            Some(format!("https://i.ytimg.com/vi/{vid}/hqdefault.jpg"))
        } else {
            None
        }
    });

    Some(OnlineTrack {
        title,
        artist,
        album,
        duration,
        thumbnail,
        url,
        source,
        reason: None,
    })
}

/// Parses yt-dlp `-J` JSON output (single object, playlist with `entries`, or NDJSON lines).
pub fn parse_ytdlp_json(raw: &str, default_source: &str) -> Vec<OnlineTrack> {
    let mut out = Vec::new();
    for chunk in std::iter::once(raw).chain(raw.lines()) {
        let trimmed = chunk.trim();
        if !trimmed.starts_with('{') {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            collect_from_value(&v, default_source, &mut out);
            if !out.is_empty() {
                break;
            }
        }
    }
    out
}

fn collect_from_value(v: &Value, default_source: &str, out: &mut Vec<OnlineTrack>) {
    if let Some(entries) = v.get("entries").and_then(|e| e.as_array()) {
        for e in entries {
            if e.get("entries").is_some() {
                collect_from_value(e, default_source, out);
            } else if let Some(t) = parse_entry(e, default_source) {
                out.push(t);
            }
        }
    } else if let Some(t) = parse_entry(v, default_source) {
        out.push(t);
    }
}

fn get_ytdlp_cmd(app: &AppHandle) -> Result<ShellCommand, String> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("yt-dlp.exe").exists() {
                return app
                    .shell()
                    .sidecar("yt-dlp")
                    .map_err(|e| format!("Không khởi tạo được yt-dlp: {e}"));
            }
        }
    }

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let _ = std::fs::create_dir_all(&data_dir);
    let bin_path = data_dir.join("yt-dlp.exe");

    if !bin_path.exists() {
        let dev_bin_dlc = PathBuf::from(
            r"c:\miu dich\dlc\src-tauri\binaries\yt-dlp-x86_64-pc-windows-msvc.exe",
        );
        let dev_bin = PathBuf::from(
            r"c:\miu dich\miu-player\src-tauri\binaries\yt-dlp-x86_64-pc-windows-msvc.exe",
        );
        if dev_bin_dlc.exists() {
            let _ = std::fs::copy(&dev_bin_dlc, &bin_path);
        } else if dev_bin.exists() {
            let _ = std::fs::copy(&dev_bin, &bin_path);
        } else {
            let resp = ureq::get("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe")
                .call()
                .map_err(|e| format!("Không tải được yt-dlp: {e}"))?;
            let mut bytes = Vec::new();
            resp.into_reader()
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            std::fs::write(&bin_path, &bytes).map_err(|e| e.to_string())?;
        }
    }

    Ok(app.shell().command(bin_path))
}

pub async fn run_ytdlp(app: &AppHandle, args: &[&str]) -> Result<String, String> {
    let mut full_args = vec![
        "--encoding",
        "utf-8",
        "--no-warnings",
        "--no-call-home",
        "--no-check-certificates",
    ];
    full_args.extend_from_slice(args);

    let cmd = get_ytdlp_cmd(app)?.args(&full_args);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Lỗi chạy yt-dlp: {e}"))?;

    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "yt-dlp kết thúc với lỗi".to_string()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn prefetch_stream_bg(app: AppHandle, url: String) {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return;
    }
    if url.contains("spotify.com") {
        return;
    }
    tauri::async_runtime::spawn(async move {
        if let Some(cache) = app.try_state::<StreamCache>() {
            let _ = resolve_stream(&app, &cache, &url).await;
        }
    });
}

pub async fn search_source(
    app: &AppHandle,
    query: &str,
    source: &str,
    limit: usize,
) -> Result<Vec<OnlineTrack>, String> {
    if source == "youtube" {
        let q_owned = query.to_string();
        if let Ok(Some(fast_tracks)) =
            tauri::async_runtime::spawn_blocking(move || fast_youtube_search(&q_owned, limit)).await
        {
            if let Some(first) = fast_tracks.first() {
                prefetch_stream_bg(app.clone(), first.url.clone());
            }
            return Ok(fast_tracks);
        }
    }
    let prefix = if source == "soundcloud" { "scsearch" } else { "ytsearch" };
    let spec = format!("{prefix}{limit}:{query}");
    let out = run_ytdlp(app, &["--flat-playlist", "-J", &spec]).await?;
    let tracks = parse_ytdlp_json(&out, source);
    if let Some(first) = tracks.first() {
        prefetch_stream_bg(app.clone(), first.url.clone());
    }
    Ok(tracks)
}

pub async fn search(
    app: &AppHandle,
    query: &str,
    source: &str,
    limit: usize,
) -> Result<Vec<OnlineTrack>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    if q.starts_with("http://") || q.starts_with("https://") {
        return resolve_info(app, q).await;
    }

    let cache_key = format!("{}:{}:{}", source, limit, q.to_lowercase());
    if let Some(sc) = app.try_state::<SearchCache>() {
        if let Some(cached) = sc.get(&cache_key) {
            if let Some(first) = cached.first() {
                prefetch_stream_bg(app.clone(), first.url.clone());
            }
            return Ok(cached);
        }
    }

    let results = match source {
        "youtube" => search_source(app, q, "youtube", limit).await?,
        "soundcloud" => search_source(app, q, "soundcloud", limit).await?,
        _ => {
            let half = (limit / 2).max(4);
            let app1 = app.clone();
            let q1 = q.to_string();
            let app2 = app.clone();
            let q2 = q.to_string();
            let h1 = tauri::async_runtime::spawn(async move {
                search_source(&app1, &q1, "youtube", half).await
            });
            let h2 = tauri::async_runtime::spawn(async move {
                search_source(&app2, &q2, "soundcloud", half).await
            });
            let yt = h1.await.ok().and_then(|r| r.ok()).unwrap_or_default();
            let sc = h2.await.ok().and_then(|r| r.ok()).unwrap_or_default();
            let mut yt_iter = yt.into_iter();
            let mut sc_iter = sc.into_iter();
            let mut merged = Vec::new();
            loop {
                let a = yt_iter.next();
                let b = sc_iter.next();
                if a.is_none() && b.is_none() {
                    break;
                }
                if let Some(x) = a {
                    merged.push(x);
                }
                if let Some(y) = b {
                    merged.push(y);
                }
            }
            merged
        }
    };

    if let Some(sc) = app.try_state::<SearchCache>() {
        sc.put(cache_key, results.clone());
    }

    Ok(results)
}

pub async fn resolve_info(app: &AppHandle, url: &str) -> Result<Vec<OnlineTrack>, String> {
    let canon = canonical_url(url);
    let cache_key = format!("info:{}", canon);
    if let Some(sc) = app.try_state::<SearchCache>() {
        if let Some(cached) = sc.get(&cache_key) {
            if let Some(first) = cached.first() {
                prefetch_stream_bg(app.clone(), first.url.clone());
            }
            return Ok(cached);
        }
    }

    // 1. Fast HTTP metadata path for single YouTube videos (~150ms vs 3500ms yt-dlp spawn)
    if extract_youtube_id(url).is_some() && !url.to_lowercase().contains("/playlist") {
        let u_owned = url.to_string();
        if let Ok(Some(track)) =
            tauri::async_runtime::spawn_blocking(move || fast_youtube_video_info(&u_owned)).await
        {
            prefetch_stream_bg(app.clone(), track.url.clone());
            let res = vec![track];
            if let Some(sc) = app.try_state::<SearchCache>() {
                sc.put(cache_key, res.clone());
            }
            return Ok(res);
        }
    }

    // 2. Fast HTTP metadata path for single SoundCloud tracks (~180ms vs 3500ms yt-dlp spawn)
    if url.to_lowercase().contains("soundcloud.com/") {
        let u_owned = url.to_string();
        if let Ok(Some(track)) =
            tauri::async_runtime::spawn_blocking(move || fast_soundcloud_track_info(&u_owned))
                .await
        {
            prefetch_stream_bg(app.clone(), track.url.clone());
            let res = vec![track];
            if let Some(sc) = app.try_state::<SearchCache>() {
                sc.put(cache_key, res.clone());
            }
            return Ok(res);
        }
    }

    // 3. Fallback to yt-dlp for playlists or other domains
    let out = run_ytdlp(
        app,
        &["--flat-playlist", "--playlist-end", "50", "-J", url],
    )
    .await?;
    let def = if url.contains("soundcloud.com") {
        "soundcloud"
    } else {
        "youtube"
    };
    let tracks = parse_ytdlp_json(&out, def);
    if let Some(first) = tracks.first() {
        prefetch_stream_bg(app.clone(), first.url.clone());
    }
    if let Some(sc) = app.try_state::<SearchCache>() {
        sc.put(cache_key, tracks.clone());
    }
    Ok(tracks)
}

pub async fn resolve_stream(
    app: &AppHandle,
    cache: &StreamCache,
    url: &str,
) -> Result<String, String> {
    let canon = canonical_url(url);
    if let Some(cached) = cache.get(&canon) {
        return Ok(cached);
    }

    // In-flight deduplication: if a background prefetch is already resolving this URL,
    // wait for it instead of spawning a second yt-dlp process.
    if !cache.begin_inflight(&canon) {
        let app_clone = app.clone();
        let canon_wait = canon.clone();
        let waited = tauri::async_runtime::spawn_blocking(move || {
            let Some(c) = app_clone.try_state::<StreamCache>() else {
                return None;
            };
            for _ in 0..240 {
                std::thread::sleep(Duration::from_millis(50));
                if let Some(hit) = c.get(&canon_wait) {
                    return Some(hit);
                }
                if !c.is_inflight(&canon_wait) {
                    break;
                }
            }
            c.get(&canon_wait)
        })
        .await
        .ok()
        .flatten();

        if let Some(stream) = waited {
            return Ok(stream);
        }
        cache.begin_inflight(&canon);
    }

    let _guard = InflightGuard {
        cache,
        key: canon.clone(),
    };

    let is_yt = extract_youtube_id(&canon).is_some();
    let out = if is_yt {
        match run_ytdlp(
            app,
            &[
                "-f",
                "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio/best",
                "--no-playlist",
                "--extractor-args",
                "youtube:player_skip=webpage,configs",
                "-g",
                &canon,
            ],
        )
        .await
        {
            Ok(s) if s.contains("http") => s,
            _ => {
                run_ytdlp(
                    app,
                    &[
                        "-f",
                        "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio/best",
                        "--no-playlist",
                        "-g",
                        &canon,
                    ],
                )
                .await?
            }
        }
    } else {
        run_ytdlp(
            app,
            &[
                "-f",
                "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio/best",
                "--no-playlist",
                "-g",
                &canon,
            ],
        )
        .await?
    };

    let stream = out
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("http://") || l.starts_with("https://"))
        .ok_or_else(|| "Không tìm được luồng phát cho bài này".to_string())?
        .to_string();

    // Chromium/WebView2 <audio> cannot play HLS .m3u8 streams (used by SoundCloud) directly;
    // cache them to a local audio file via yt-dlp's native HLS downloader.
    if stream.contains(".m3u8") {
        let cache_dir = app
            .path()
            .app_cache_dir()
            .map_err(|e| e.to_string())?
            .join("stream_cache");
        std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
        let hash = format!("{:x}", Sha256::digest(canon.as_bytes()));
        let out_tpl = cache_dir
            .join(format!("{}.%(ext)s", &hash[..16]))
            .to_string_lossy()
            .to_string();
        let printed = run_ytdlp(
            app,
            &[
                "-f",
                "bestaudio[ext=m4a]/bestaudio[ext=mp3]/bestaudio/best",
                "--no-playlist",
                "--print",
                "after_move:FILE:%(filepath)s",
                "-o",
                &out_tpl,
                &canon,
            ],
        )
        .await?;
        if let Some(fp) = printed
            .lines()
            .find_map(|l| l.trim().strip_prefix("FILE:"))
        {
            let local = fp.trim().to_string();
            cache.put(url.to_string(), local.clone());
            return Ok(local);
        }
    }

    cache.put(url.to_string(), stream.clone());
    Ok(stream)
}

/// Downloads a remote thumbnail into `cover_dir` for offline display.
pub fn cache_thumbnail(cover_dir: &Path, thumb_url: &str) -> Option<String> {
    if !thumb_url.starts_with("http") {
        return None;
    }
    let hash = format!("{:x}", Sha256::digest(thumb_url.as_bytes()));
    let file = cover_dir.join(format!("online_{}.jpg", &hash[..16]));
    if file.exists() {
        return Some(file.to_string_lossy().to_string());
    }
    let resp = http_agent().get(thumb_url).call().ok()?;
    let mut bytes = Vec::new();
    resp.into_reader().take(5 * 1024 * 1024).read_to_end(&mut bytes).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let _ = std::fs::create_dir_all(cover_dir);
    std::fs::write(&file, &bytes).ok()?;
    Some(file.to_string_lossy().to_string())
}

pub async fn download_audio(
    app: &AppHandle,
    url: &str,
    dest_dir: &Path,
    track_id: Option<i64>,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    let template = dest_dir
        .join("%(title).100B [%(id)s].%(ext)s")
        .to_string_lossy()
        .to_string();

    let args = [
        "--encoding",
        "utf-8",
        "--no-warnings",
        "--no-playlist",
        "--newline",
        "--progress-template",
        "PROG:%(progress._percent_str)s",
        "-f",
        "bestaudio[ext=m4a]/bestaudio[ext=opus]/bestaudio[ext=webm]/bestaudio/best",
        "--print",
        "after_move:FILE:%(filepath)s",
        "-o",
        &template,
        url,
    ];

    let (mut rx, _child) = get_ytdlp_cmd(app)?
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            url: url.to_string(),
            track_id,
            percent: 0.0,
            status: "downloading".into(),
        },
    );

    let mut final_path: Option<PathBuf> = None;
    let mut err_lines = Vec::new();

    while let Some(ev) = rx.recv().await {
        match ev {
            CommandEvent::Stdout(bytes) => {
                for line in String::from_utf8_lossy(&bytes).lines() {
                    let s = line.trim();
                    if let Some(rest) = s.strip_prefix("PROG:") {
                        let cleaned = rest.trim().trim_end_matches('%').trim();
                        if let Ok(p) = cleaned.parse::<f64>() {
                            let _ = app.emit(
                                "download-progress",
                                DownloadProgress {
                                    url: url.to_string(),
                                    track_id,
                                    percent: p,
                                    status: "downloading".into(),
                                },
                            );
                        }
                    } else if let Some(fp) = s.strip_prefix("FILE:") {
                        final_path = Some(PathBuf::from(fp.trim()));
                    }
                }
            }
            CommandEvent::Stderr(bytes) => {
                let s = String::from_utf8_lossy(&bytes).trim().to_string();
                if !s.is_empty() {
                    err_lines.push(s);
                }
            }
            CommandEvent::Terminated(_) => break,
            _ => {}
        }
    }

    match final_path {
        Some(p) if p.exists() => {
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    url: url.to_string(),
                    track_id,
                    percent: 100.0,
                    status: "done".into(),
                },
            );
            Ok(p)
        }
        _ => {
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    url: url.to_string(),
                    track_id,
                    percent: 0.0,
                    status: "error".into(),
                },
            );
            Err(if err_lines.is_empty() {
                "Tải file thất bại".to_string()
            } else {
                err_lines.join("\n")
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_search_entries_and_builds_youtube_url() {
        let json = r#"{
            "entries": [
                {
                    "id": "dQw4w9WgXcQ",
                    "title": "Never Gonna Give You Up",
                    "uploader": "Rick Astley",
                    "duration": 212.0,
                    "ie_key": "Youtube"
                },
                {
                    "id": "sc123",
                    "title": "Chill Track",
                    "uploader": "LoFi Producer",
                    "duration": 145.5,
                    "webpage_url": "https://soundcloud.com/lofi/chill-track",
                    "thumbnail": "https://i1.sndcdn.com/artworks-1.jpg",
                    "extractor_key": "Soundcloud"
                },
                {
                    "id": "del",
                    "title": "[Deleted video]"
                }
            ]
        }"#;
        let tracks = parse_ytdlp_json(json, "youtube");
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].title, "Never Gonna Give You Up");
        assert_eq!(tracks[0].artist, "Rick Astley");
        assert_eq!(tracks[0].source, "youtube");
        assert_eq!(tracks[0].url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(
            tracks[0].thumbnail.as_deref(),
            Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg")
        );
        assert_eq!(tracks[1].source, "soundcloud");
        assert_eq!(tracks[1].album, "SoundCloud");
    }

    #[test]
    fn parses_single_video_json() {
        let json = r#"{
            "id": "abc123xyz",
            "title": "Bài Hát Việt",
            "artist": "Ca Sĩ A",
            "album": "Single 2026",
            "duration": 240,
            "webpage_url": "https://www.youtube.com/watch?v=abc123xyz",
            "extractor_key": "Youtube"
        }"#;
        let tracks = parse_ytdlp_json(json, "youtube");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].artist, "Ca Sĩ A");
        assert_eq!(tracks[0].album, "Single 2026");
    }
}
