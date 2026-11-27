use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MAX_LYRICS: u64 = 512 * 1024;
pub const UNRESOLVED_MARKER: &str = "⟦không nghe rõ⟧";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LyricsQuery {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct LyricsResult {
    pub content: String,
    pub source: String,
    pub timing: String,
    pub review: bool,
    #[serde(default)]
    pub draft_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

impl LyricsResult {
    pub fn new(content: String, source: &str, review: bool) -> Self {
        let has_line = content.lines().any(|line| {
            line.starts_with('[')
                && line
                    .split_once(']')
                    .is_some_and(|(tag, _)| lrc_time(&tag[1..]).is_some())
        });
        let timing = if has_line
            && content.split('<').skip(1).any(|part| {
                part.split_once('>')
                    .is_some_and(|(tag, text)| lrc_time(tag).is_some() && !text.trim().is_empty())
            }) {
            "word"
        } else if has_line {
            "line"
        } else {
            "plain"
        };
        Self {
            content,
            source: source.into(),
            timing: timing.into(),
            review,
            draft_only: false,
            notice: None,
        }
    }
}

pub fn key(q: &LyricsQuery) -> String {
    let file = std::fs::metadata(&q.path).ok().filter(|m| m.is_file());
    let size = file.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = file
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let identity = format!(
        "{}\0{}\0{}\0{}\0{size}\0{modified}",
        q.path, q.title, q.artist, q.album
    );
    format!("{:x}", Sha256::digest(identity.as_bytes()))
}

fn legacy_file(dir: &Path, q: &LyricsQuery) -> PathBuf {
    let key = format!("{}|{}|{}", q.artist, q.title, q.album).to_lowercase();
    let hash = format!("{:x}", Sha256::digest(key.as_bytes()));
    dir.join(format!("{}.lrc", &hash[..24]))
}

pub fn read_limited(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > max {
        return Err("Tệp vượt giới hạn dung lượng.".into());
    }
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("Tệp vượt giới hạn dung lượng.".into());
    }
    Ok(bytes)
}

pub fn read_text(path: &Path) -> Option<String> {
    let text = String::from_utf8(read_limited(path, MAX_LYRICS).ok()?).ok()?;
    (!text.trim().is_empty()).then_some(text)
}

pub fn atomic_json<T: Serialize>(path: &Path, value: &T, no_clobber: bool) -> Result<(), String> {
    let dir = path.parent().ok_or("Đường dẫn lưu không hợp lệ.")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, value).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    if no_clobber {
        match file.persist_noclobber(path) {
            Ok(_) => Ok(()),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e.error.to_string()),
        }
    } else {
        file.persist(path)
            .map(|_| ())
            .map_err(|e| e.error.to_string())
    }
}

fn cached(path: &Path) -> Option<LyricsResult> {
    let item: LyricsResult = serde_json::from_slice(&read_limited(path, MAX_LYRICS).ok()?).ok()?;
    (!item.content.trim().is_empty()).then_some(item)
}

pub fn deleted(dir: &Path, q: &LyricsQuery) -> bool {
    let id = key(q);
    dir.join(format!("{id}.deleted.json")).exists()
        && cached(&dir.join(format!("{id}.manual.json"))).is_none()
}

pub fn delete_saved(dir: &Path, q: &LyricsQuery) -> Result<(), String> {
    let id = key(q);
    atomic_json(&dir.join(format!("{id}.deleted.json")), &true, false)?;
    for kind in ["manual", "auto", "draft"] {
        match std::fs::remove_file(dir.join(format!("{id}.{kind}.json"))) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(
                    "Chưa xóa hết cache lời trong app. Hãy thử lại. File .lrc gốc không bị xóa."
                        .into(),
                );
            }
        }
    }
    Ok(())
}

pub fn local(dir: &Path, q: &LyricsQuery) -> Option<LyricsResult> {
    let id = key(q);
    if let Some(item) = cached(&dir.join(format!("{id}.manual.json"))) {
        return Some(item);
    }
    if dir.join(format!("{id}.deleted.json")).exists() {
        return None;
    }
    if !q.path.starts_with("http://")
        && !q.path.starts_with("https://")
        && let Some(text) = read_text(&Path::new(&q.path).with_extension("lrc"))
    {
        return Some(LyricsResult::new(text, "lrc", false));
    }
    cached(&dir.join(format!("{id}.auto.json")))
        .or_else(|| {
            read_text(&legacy_file(dir, q)).map(|s| LyricsResult::new(s, "legacy-cache", true))
        })
        .or_else(|| cached(&dir.join(format!("{id}.draft.json"))).filter(|item| item.draft_only))
}

fn save_cached(path: &Path, item: &LyricsResult, no_clobber: bool) -> Result<(), String> {
    if serde_json::to_vec(item).map_err(|e| e.to_string())?.len() as u64 > MAX_LYRICS {
        return Err("Lời bài hát sau khi mã hóa vượt giới hạn; bản cũ được giữ nguyên.".into());
    }
    atomic_json(path, item, no_clobber)
}

pub fn save_manual(dir: &Path, q: &LyricsQuery, content: &str) -> Result<(), String> {
    if content.contains(UNRESOLVED_MARKER) {
        return Err(
            "Còn vùng chưa nghe rõ. Nghe và sửa các dấu đánh dấu trước khi lưu lyric hoàn chỉnh."
                .into(),
        );
    }
    if content.trim().is_empty() || content.len() as u64 > MAX_LYRICS / 2 {
        return Err("Lời bài hát trống hoặc quá dài.".into());
    }
    save_cached(
        &dir.join(format!("{}.manual.json", key(q))),
        &LyricsResult::new(content.into(), "manual", false),
        false,
    )
}

pub fn save_generated(
    dir: &Path,
    q: &LyricsQuery,
    item: &LyricsResult,
) -> Result<LyricsResult, String> {
    if let Some(existing) = local(dir, q) {
        return Ok(existing);
    }
    if deleted(dir, q) {
        return Err(
            "Lời đã được xóa. Bấm Viết lại bằng AI hoặc nhập lời rồi lưu để tạo bản mới.".into(),
        );
    }
    let kind = if item.draft_only { "draft" } else { "auto" };
    save_cached(&dir.join(format!("{}.{kind}.json", key(q))), item, true)?;
    local(dir, q).ok_or_else(|| "Không thể đọc lại lời đã lưu.".into())
}

pub fn clean_title(title: &str) -> String {
    let mut out = title.to_string();
    for pattern in [
        "| official music video",
        "| official video",
        "| official audio",
        "| lyric video",
        "[official music video]",
        "[official video]",
        "[official audio]",
        "[official mv]",
        "(official music video)",
        "(official video)",
        "(official audio)",
        "(official lyric video)",
        "(official visualizer)",
        "(official mv)",
        "[mv]",
        "(mv)",
        "[lyrics]",
        "(lyrics)",
        "[audio]",
        "(audio)",
    ] {
        // ASCII-only case folding preserves UTF-8 byte offsets for replacement.
        while let Some(index) = out.to_ascii_lowercase().find(pattern) {
            out.replace_range(index..index + pattern.len(), "");
        }
    }
    out.trim().to_string()
}

pub(crate) fn normalized(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn matches_record(q: &LyricsQuery, v: &Value) -> bool {
    let title = clean_title(&q.title);
    let (artist, title) = if ["", "Unknown Artist", "Nghệ sĩ chưa rõ"].contains(&q.artist.as_str())
    {
        match title.split_once(" - ") {
            Some((a, t)) => (a, t),
            None => return false,
        }
    } else {
        (q.artist.as_str(), title.as_str())
    };
    normalized(title) == normalized(v["trackName"].as_str().unwrap_or(""))
        && normalized(artist) == normalized(v["artistName"].as_str().unwrap_or(""))
        && q.duration.is_finite()
        && q.duration > 0.0
        && v["duration"]
            .as_f64()
            .is_some_and(|d| (q.duration - d).abs() <= 3.0)
}

pub fn pick_lyrics(v: &Value) -> Option<String> {
    ["syncedLyrics", "plainLyrics"].iter().find_map(|k| {
        v[k].as_str()
            .filter(|s| !s.trim().is_empty() && s.len() as u64 <= MAX_LYRICS / 2)
            .map(str::to_string)
    })
}

pub(crate) fn lrc_time(text: &str) -> Option<f64> {
    let (minutes, seconds) = text.split_once(':')?;
    if minutes.is_empty() || minutes.len() > 3 || !minutes.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let seconds = seconds.replace(':', ".");
    let (whole, fraction) = seconds.split_once('.').unwrap_or((&seconds, ""));
    if whole.is_empty()
        || whole.len() > 2
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || fraction.len() > 3
        || !fraction.bytes().all(|c| c.is_ascii_digit())
        || (seconds.contains('.') && fraction.is_empty())
    {
        return None;
    }
    let seconds: f64 = seconds.parse().ok()?;
    (seconds.is_finite() && (0.0..60.0).contains(&seconds))
        .then(|| minutes.parse::<f64>().unwrap() * 60.0 + seconds)
}

fn timing_rank(item: &LyricsResult) -> u8 {
    match item.timing.as_str() {
        "word" => 2,
        "line" => 1,
        _ => 0,
    }
}

pub fn fetch_online(
    q: &LyricsQuery,
    lyricsplus: bool,
    language: &str,
) -> Result<Option<LyricsResult>, String> {
    if !lyricsplus {
        return fetch_lrclib(q);
    }
    std::thread::scope(|scope| {
        let lrclib = scope.spawn(|| fetch_lrclib(q));
        let plus = scope.spawn(|| fetch_lyricsplus(q, language));
        let lrclib = lrclib.join().map_err(|_| "Tra cứu LRCLIB bị gián đoạn.")?;
        let plus = plus
            .join()
            .map_err(|_| "Tra cứu LyricsPlus bị gián đoạn.")?;
        // Metadata must match before quality is considered. On ties keep the existing LRCLIB source.
        Ok([plus.ok().flatten(), lrclib.ok().flatten()]
            .into_iter()
            .flatten()
            .max_by_key(timing_rank))
    })
}

fn fetch_lrclib(q: &LyricsQuery) -> Result<Option<LyricsResult>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .user_agent("dlc/0.1 lyrics")
        .build();
    let cleaned = clean_title(&q.title);
    if cleaned.is_empty() {
        return Ok(None);
    }
    let artist = if ["Unknown Artist", "Nghệ sĩ chưa rõ"].contains(&q.artist.as_str()) {
        ""
    } else {
        &q.artist
    };
    let response = agent
        .get("https://lrclib.net/api/search")
        .query("q", &format!("{} {}", cleaned, artist))
        .call()
        .map_err(|_| "Không kết nối được LRCLIB.")?;
    let data = online_json(response)?;
    let Some(items) = data.as_array() else {
        return Ok(None);
    };
    Ok(items
        .iter()
        .filter(|v| matches_record(q, v))
        .filter_map(pick_lyrics)
        .map(|content| LyricsResult::new(content, "lrclib", false))
        .max_by_key(timing_rank))
}

fn online_json(response: ureq::Response) -> Result<Value, String> {
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_LYRICS * 8 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Không đọc được phản hồi nguồn lời.")?;
    if bytes.len() as u64 > MAX_LYRICS * 8 {
        return Err("Phản hồi nguồn lời quá lớn.".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "Nguồn lời không trả JSON hợp lệ.".into())
}

fn fetch_lyricsplus(q: &LyricsQuery, language: &str) -> Result<Option<LyricsResult>, String> {
    let title = clean_title(&q.title);
    let (artist, title) = if ["", "Unknown Artist", "Nghệ sĩ chưa rõ"].contains(&q.artist.as_str())
    {
        let Some((artist, title)) = title.split_once(" - ") else {
            return Ok(None);
        };
        (artist, title)
    } else {
        (q.artist.as_str(), title.as_str())
    };
    if title.trim().is_empty() || !q.duration.is_finite() || q.duration <= 0.0 {
        return Ok(None);
    }
    let response = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .redirects(0)
        .user_agent("dlc/0.1 lyrics")
        .build()
        .get("https://lyricsplus.binimum.org/v2/lyrics/get")
        .query("title", title)
        .query("artist", artist)
        .query("duration", &q.duration.to_string())
        .query("album", &q.album)
        .call()
        .map_err(|_| "Không kết nối được LyricsPlus.")?;
    lyricsplus_result(&online_json(response)?, q, language)
}

fn lyricsplus_result(
    v: &Value,
    q: &LyricsQuery,
    language: &str,
) -> Result<Option<LyricsResult>, String> {
    let metadata = &v["metadata"];
    let duration = metadata["totalDuration"].as_str().and_then(|s| {
        let (minutes, seconds) = s.split_once(':')?;
        let minutes: u32 = minutes.parse().ok()?;
        let seconds: f64 = seconds.parse().ok()?;
        (seconds.is_finite() && (0.0..60.0).contains(&seconds))
            .then_some(minutes as f64 * 60.0 + seconds)
    });
    let record = serde_json::json!({"trackName":metadata["title"],"artistName":metadata["artist"],"duration":duration});
    if !matches_record(q, &record)
        || (language != "auto"
            && metadata["language"]
                .as_str()
                .and_then(|s| s.split('-').next())
                != Some(language))
    {
        return Ok(None);
    }
    let Some(lines) = v["lyrics"].as_array() else {
        return Ok(None);
    };
    let mut segments = Vec::new();
    for line in lines {
        let text = safe_text(line["text"].as_str().unwrap_or(""));
        if text.is_empty() {
            continue;
        }
        let start = line["time"].as_f64().ok_or("LyricsPlus thiếu mốc câu.")? / 1000.0;
        let span = line["duration"]
            .as_f64()
            .ok_or("LyricsPlus thiếu thời lượng câu.")?
            / 1000.0;
        if !span.is_finite() || span <= 0.0 {
            return Err("Thời lượng câu LyricsPlus không hợp lệ.".into());
        }
        let end = start + span;
        let words = lyricsplus_words(line, &text, start, end);
        segments.push(serde_json::json!({"start":start,"end":end,"text":text,"words":words}));
    }
    Ok(Some(LyricsResult::new(
        whisper_to_lrc(&serde_json::json!({"segments":segments}), q.duration)?,
        "lyricsplus",
        true,
    )))
}

// Match timed fragments to the original words, so "to" + "gether" never becomes "to gether".
fn lyricsplus_words(line: &Value, text: &str, start: f64, end: f64) -> Option<Vec<Value>> {
    let mut pieces = Vec::new();
    let mut previous = start;
    for syllable in line["syllabus"].as_array()? {
        let ws = syllable["time"].as_f64()? / 1000.0;
        let span = syllable["duration"].as_f64()? / 1000.0;
        let we = ws + span;
        if !ws.is_finite()
            || !span.is_finite()
            || span <= 0.0
            || ws < previous - 0.05
            || we > end + 0.05
        {
            return None;
        }
        let piece = normalized(syllable["text"].as_str()?);
        if we <= ws.max(previous) {
            return None;
        }
        if !piece.is_empty() {
            pieces.push((ws.max(previous), we, piece));
        }
        previous = we;
    }
    let mut index = 0;
    let mut words = Vec::new();
    for word in text.split_whitespace() {
        let target = normalized(word);
        if target.is_empty() {
            return None;
        }
        let first = pieces.get(index)?.0;
        let mut remaining = target.as_str();
        let mut last = first;
        while !remaining.is_empty() {
            let piece = pieces.get(index)?;
            remaining = remaining.strip_prefix(piece.2.as_str())?;
            last = piece.1;
            index += 1;
        }
        words.push(serde_json::json!({"word":word,"start":first,"end":last}));
    }
    (index == pieces.len()).then_some(words)
}

pub fn stamp(seconds: f64) -> Result<String, String> {
    if !seconds.is_finite() || !(0.0..=86400.0).contains(&seconds) {
        return Err("Mốc thời gian không hợp lệ.".into());
    }
    let cs = (seconds * 100.0).round() as u64;
    Ok(format!(
        "{:02}:{:02}.{:02}",
        cs / 6000,
        cs / 100 % 60,
        cs % 100
    ))
}

fn safe_text(text: &str) -> String {
    text.replace(['\r', '\n', '<', '>', '[', ']'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn whisper_to_lrc(v: &Value, duration: f64) -> Result<String, String> {
    if !duration.is_finite() || duration <= 0.0 {
        return Err("Thời lượng bài hát không hợp lệ.".into());
    }
    let mut lines = Vec::new();
    let mut previous = 0.0;
    let bound = duration + 1.0;
    let top_words = v["words"].as_array();
    if let Some(segments) = v["segments"].as_array() {
        for seg in segments {
            if seg["no_speech_prob"].as_f64().is_some_and(|p| p > 0.8)
                || seg["avg_logprob"].as_f64().is_some_and(|p| p < -1.5)
                || seg["compression_ratio"].as_f64().is_some_and(|r| r > 2.4)
            {
                continue;
            }
            let text = safe_text(seg["text"].as_str().unwrap_or(""));
            if text.is_empty() {
                continue;
            }
            let start = seg["start"].as_f64().ok_or("Thiếu thời gian câu hát.")?;
            let end = seg["end"].as_f64().ok_or("Thiếu thời gian kết thúc.")?;
            if !start.is_finite()
                || !end.is_finite()
                || start < previous
                || end < start
                || end > bound
            {
                return Err("Thời gian AI nằm ngoài bài hát.".into());
            }
            let words = seg["words"].as_array().or(top_words);
            let mut timed_words = Vec::new();
            let mut last_end = start;
            if let Some(words) = words {
                for word in words {
                    let ws = word["start"].as_f64().ok_or("Thiếu thời gian từ.")?;
                    let we = word["end"].as_f64().ok_or("Thiếu thời gian từ.")?;
                    if ws < start || ws >= end {
                        continue;
                    }
                    let wt = safe_text(word["word"].as_str().unwrap_or(""));
                    if wt.is_empty() {
                        continue;
                    }
                    if !ws.is_finite()
                        || !we.is_finite()
                        || ws < last_end - 0.05
                        || we < ws
                        || we > end + 0.05
                        || we > bound
                    {
                        return Err("Mốc từ AI không hợp lệ.".into());
                    }
                    if we <= ws.max(last_end) {
                        continue;
                    }
                    timed_words.push((ws.max(last_end), we, wt));
                    last_end = we;
                }
            }
            let spelling: Vec<_> = text.split_whitespace().collect();
            if !timed_words.is_empty()
                && spelling.len() == timed_words.len()
                && spelling
                    .iter()
                    .zip(&timed_words)
                    .all(|(text, (_, _, word))| normalized(text) == normalized(word))
            {
                let mut parts = Vec::new();
                last_end = start;
                for ((ws, we, _), word) in timed_words.iter().zip(spelling) {
                    if last_end < *ws && !parts.is_empty() {
                        parts.push(format!("<{}>", stamp(last_end)?));
                    }
                    // Segment text retains capitalization and punctuation omitted by word timestamps.
                    parts.push(format!("<{}>{word}", stamp(*ws)?));
                    last_end = *we;
                }
                lines.push(format!(
                    "[{}]{}<{}>",
                    stamp(start)?,
                    parts.join(" "),
                    stamp(last_end)?
                ));
            } else {
                lines.push(format!("[{}]{text}", stamp(start)?));
            }
            previous = start;
        }
    }
    if lines.is_empty() {
        return Err("Không tìm thấy giọng hát có thời gian hợp lệ; không lưu lời giả.".into());
    }
    let text = lines.join("\n");
    if text.len() as u64 > MAX_LYRICS / 2 {
        return Err("Lời AI quá dài.".into());
    }
    Ok(text)
}

pub fn captions_to_lrc(v: &Value, duration: f64) -> Result<String, String> {
    if !duration.is_finite() || duration <= 0.0 {
        return Err("Thời lượng bài hát không hợp lệ.".into());
    }
    let events = v["events"].as_array().ok_or("Phụ đề JSON3 không hợp lệ.")?;
    let mut lines = Vec::new();
    let mut last = String::new();
    let mut previous_time = 0.0;
    let mut previous_end = 0.0;
    for event in events {
        let Some(segs) = event["segs"].as_array() else {
            continue;
        };
        let Some(ms) = event["tStartMs"].as_f64() else {
            continue;
        };
        let start = ms / 1000.0;
        if !start.is_finite() || start < previous_time || start > duration + 1.0 {
            continue;
        }
        let text = safe_text(
            &segs
                .iter()
                .filter_map(|s| s["utf8"].as_str())
                .collect::<String>(),
        );
        if text.is_empty() {
            continue;
        }
        let current: Vec<&str> = text.split_whitespace().collect();
        let prior: Vec<&str> = last.split_whitespace().collect();
        // Never erase an identical phrase at a new onset, even if display windows overlap.
        // ponytail: growing untimed windows use text overlap; absolute word offsets would resolve ambiguous partial repeats.
        let overlap = if start < previous_end
            && event["aAppend"].as_u64() != Some(1)
            && (text != last || start == previous_time)
        {
            (1..=prior.len().min(current.len()))
                .rev()
                .find(|&n| prior[prior.len() - n..] == current[..n])
                .unwrap_or(0)
        } else {
            0
        };
        let end = event["dDurationMs"]
            .as_f64()
            .filter(|n| n.is_finite() && *n > 0.0)
            .map(|n| start + n / 1000.0)
            .unwrap_or(start);
        let fresh = current[overlap..].join(" ");
        if !fresh.is_empty() {
            let mut words = Vec::new();
            let mut precise = end > start && end <= duration + 1.0;
            for (i, seg) in segs.iter().enumerate() {
                let word = safe_text(seg["utf8"].as_str().unwrap_or(""));
                if word.is_empty() {
                    continue;
                }
                // JSON3 omits the first offset when it is zero.
                let offset = seg["tOffsetMs"]
                    .as_f64()
                    .or_else(|| words.is_empty().then_some(0.0));
                let next = segs[i + 1..]
                    .iter()
                    .find(|s| !safe_text(s["utf8"].as_str().unwrap_or("")).is_empty());
                let ws = offset.map(|n| start + n / 1000.0).unwrap_or(start);
                let we = next
                    .map(|s| {
                        s["tOffsetMs"]
                            .as_f64()
                            .map(|n| start + n / 1000.0)
                            .unwrap_or(ws)
                    })
                    .unwrap_or(end);
                precise &= offset.is_some()
                    && word.split_whitespace().count() == 1
                    && ws.is_finite()
                    && we.is_finite()
                    && ws >= start
                    && we > ws
                    && we <= end;
                words.push(serde_json::json!({"start":ws,"end":we,"word":word}));
            }
            precise &= words.len() == current.len();
            if precise {
                let words = &words[overlap..];
                let segment = serde_json::json!({"start":words[0]["start"],"end":end,"text":fresh,"words":words});
                lines.push(whisper_to_lrc(
                    &serde_json::json!({"segments":[segment]}),
                    duration,
                )?);
            } else {
                lines.push(format!("[{}]{fresh}", stamp(start)?));
            }
        }
        last = text;
        previous_time = start;
        previous_end = end;
    }
    if lines.is_empty() {
        return Err("Phụ đề không chứa lời có thời gian hợp lệ.".into());
    }
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn query() -> LyricsQuery {
        LyricsQuery {
            path: "track.mp3".into(),
            title: "İ Song (Official Audio)".into(),
            artist: "Artist".into(),
            album: "".into(),
            duration: 120.0,
            url: None,
        }
    }
    #[test]
    fn matching_is_conservative_and_unicode_safe() {
        assert_eq!(clean_title("İ Song (Official Audio)"), "İ Song");
        let q = query();
        let mut v =
            serde_json::json!({"trackName":"İ Song", "artistName":"Artist", "duration":120});
        assert!(matches_record(&q, &v));
        v["duration"] = 140.into();
        assert!(!matches_record(&q, &v));
        v["duration"] = 120.into();
        v["artistName"] = "Other".into();
        assert!(!matches_record(&q, &v));
    }
    fn lyricsplus_fixture() -> Value {
        serde_json::json!({"type":"Word","metadata":{"title":"İ Song","artist":"Artist","totalDuration":"2:00.000","language":"en"},"lyrics":[{
            "time":1000,"duration":3500,"text":"Together, la la!","syllabus":[
                {"time":1000,"duration":100,"text":"To"},
                {"time":1100,"duration":400,"text":"gether, "},
                {"time":2000,"duration":100,"text":"la "},
                {"time":3000,"duration":1500,"text":"la!"}
            ]
        }]})
    }
    #[test]
    fn lyricsplus_joins_fragments_without_changing_spelling_or_repeats() {
        let q = query();
        let value = lyricsplus_fixture();
        let result = lyricsplus_result(&value, &q, "auto").unwrap().unwrap();
        assert_eq!(result.source, "lyricsplus");
        assert_eq!(result.timing, "word");
        assert_eq!(
            result.content,
            "[00:01.00]<00:01.00>Together, <00:01.50> <00:02.00>la <00:02.10> <00:03.00>la!<00:04.50>"
        );
        let mut broken = value.clone();
        broken["lyrics"][0]["syllabus"][1]["text"] = serde_json::json!("wrong");
        let fallback = lyricsplus_result(&broken, &q, "auto").unwrap().unwrap();
        assert_eq!(fallback.timing, "line");
        assert_eq!(fallback.content, "[00:01.00]Together, la la!");
        broken["lyrics"][0]["time"] = serde_json::json!(-1000);
        assert!(lyricsplus_result(&broken, &q, "auto").is_err());
    }
    #[test]
    fn lyricsplus_rejects_wrong_recording_language_or_missing_identity() {
        let q = query();
        let value = lyricsplus_fixture();
        assert!(lyricsplus_result(&value, &q, "vi").unwrap().is_none());
        assert!(lyricsplus_result(&value, &q, "en").unwrap().is_some());
        for (field, bad) in [
            ("title", "Another song"),
            ("artist", "Cover artist"),
            ("totalDuration", "3:00"),
            ("totalDuration", "invalid"),
        ] {
            let mut wrong = value.clone();
            wrong["metadata"][field] = serde_json::json!(bad);
            assert!(lyricsplus_result(&wrong, &q, "auto").unwrap().is_none());
        }
        let mut wrong = value.clone();
        wrong["metadata"].as_object_mut().unwrap().remove("artist");
        assert!(lyricsplus_result(&wrong, &q, "auto").unwrap().is_none());
    }
    #[test]
    fn timing_quality_requires_real_timestamps_not_emoticons() {
        let plain = LyricsResult::new("[ti:Song]\nLove <3>".into(), "test", false);
        let line = LyricsResult::new("[00:01.00]Love <3>".into(), "test", false);
        let word = LyricsResult::new("[00:01.00]<00:01.00>Love<00:02.00>".into(), "test", false);
        assert_eq!(plain.timing, "plain");
        assert_eq!(line.timing, "line");
        assert_eq!(word.timing, "word");
        assert!(timing_rank(&word) > timing_rank(&line));
        assert!(timing_rank(&line) > timing_rank(&plain));
    }
    #[test]
    fn manual_and_sidecar_survive_automatic_generation() {
        let dir = tempfile::tempdir().unwrap();
        let mut q = query();
        q.path = dir.path().join("song.mp3").to_string_lossy().into();
        let sidecar = Path::new(&q.path).with_extension("lrc");
        std::fs::write(&sidecar, "[00:01.00]Original").unwrap();
        let ai = LyricsResult::new("[00:01.00]AI".into(), "groq", true);
        assert_eq!(save_generated(dir.path(), &q, &ai).unwrap().source, "lrc");
        save_manual(dir.path(), &q, "[00:02.00]Edited").unwrap();
        assert_eq!(local(dir.path(), &q).unwrap().source, "manual");
        assert_eq!(
            std::fs::read_to_string(sidecar).unwrap(),
            "[00:01.00]Original"
        );
        assert_eq!(
            save_generated(dir.path(), &q, &ai).unwrap().source,
            "manual"
        );
        let other = LyricsQuery {
            path: "another.mp3".into(),
            ..q
        };
        assert!(local(dir.path(), &other).is_none());
    }
    #[test]
    fn uncertain_drafts_are_separate_reusable_and_cannot_replace_confirmed_lyrics() {
        let dir = tempfile::tempdir().unwrap();
        let q = query();
        let mut draft = LyricsResult::new(format!("Xin {UNRESOLVED_MARKER}"), "gemini", true);
        draft.draft_only = true;
        let first = save_generated(dir.path(), &q, &draft).unwrap();
        assert!(first.draft_only);
        assert!(dir.path().join(format!("{}.draft.json", key(&q))).exists());
        assert!(!dir.path().join(format!("{}.auto.json", key(&q))).exists());
        assert_eq!(local(dir.path(), &q).unwrap(), first);
        assert!(save_manual(dir.path(), &q, &draft.content).is_err());
        save_manual(dir.path(), &q, "Xin chào!").unwrap();
        assert!(!local(dir.path(), &q).unwrap().draft_only);
        assert_eq!(
            save_generated(dir.path(), &q, &draft).unwrap().content,
            "Xin chào!"
        );
        delete_saved(dir.path(), &q).unwrap();
        assert!(local(dir.path(), &q).is_none());
        assert!(!dir.path().join(format!("{}.draft.json", key(&q))).exists());
        let old: LyricsResult = serde_json::from_str(
            r#"{"content":"Existing","source":"manual","timing":"plain","review":false}"#,
        )
        .unwrap();
        assert!(!old.draft_only);
    }
    #[test]
    fn deleting_lyrics_suppresses_fallbacks_and_manual_save_restores() {
        let dir = tempfile::tempdir().unwrap();
        let mut q = query();
        q.path = dir.path().join("song.mp3").to_string_lossy().into();
        let sidecar = Path::new(&q.path).with_extension("lrc");
        std::fs::write(&sidecar, "Original sidecar").unwrap();
        std::fs::write(legacy_file(dir.path(), &q), "Legacy cache").unwrap();
        let ai = LyricsResult::new("Generated".into(), "groq", true);
        save_cached(
            &dir.path().join(format!("{}.auto.json", key(&q))),
            &ai,
            false,
        )
        .unwrap();
        save_manual(dir.path(), &q, "Edited").unwrap();
        let other = LyricsQuery {
            path: "other-song.mp3".into(),
            ..q.clone()
        };
        save_manual(dir.path(), &other, "Other song").unwrap();
        delete_saved(dir.path(), &q).unwrap();
        assert!(deleted(dir.path(), &q));
        assert!(local(dir.path(), &q).is_none());
        for kind in ["manual", "auto"] {
            assert!(!dir.path().join(format!("{}.{kind}.json", key(&q))).exists());
        }
        assert_eq!(
            std::fs::read_to_string(&sidecar).unwrap(),
            "Original sidecar"
        );
        assert_eq!(local(dir.path(), &other).unwrap().content, "Other song");
        assert!(save_generated(dir.path(), &q, &ai).is_err());
        assert!(local(dir.path(), &q).is_none());
        delete_saved(dir.path(), &q).unwrap();
        assert!(save_manual(dir.path(), &q, "").is_err());
        assert!(deleted(dir.path(), &q));
        save_manual(dir.path(), &q, "Rewritten").unwrap();
        assert!(!deleted(dir.path(), &q));
        assert_eq!(local(dir.path(), &q).unwrap().content, "Rewritten");
        delete_saved(dir.path(), &q).unwrap();
        assert!(local(dir.path(), &q).is_none());
    }
    #[test]
    fn failed_delete_does_not_touch_unrelated_files() {
        let dir = tempfile::tempdir().unwrap();
        let q = query();
        let blocked = dir.path().join(format!("{}.manual.json", key(&q)));
        std::fs::create_dir(&blocked).unwrap();
        std::fs::write(blocked.join("keep.txt"), "Keep").unwrap();
        assert!(delete_saved(dir.path(), &q).is_err());
        assert_eq!(
            std::fs::read_to_string(blocked.join("keep.txt")).unwrap(),
            "Keep"
        );
    }
    #[test]
    fn oversized_json_does_not_replace_saved_lyrics() {
        let dir = tempfile::tempdir().unwrap();
        let q = query();
        save_manual(dir.path(), &q, "Original lyrics").unwrap();
        assert!(save_manual(dir.path(), &q, &"\u{0001}".repeat(100_000)).is_err());
        assert_eq!(local(dir.path(), &q).unwrap().content, "Original lyrics");
        let other = LyricsQuery {
            path: "other.wav".into(),
            ..q
        };
        let bad = LyricsResult::new("\u{0001}".repeat(100_000), "gemini", true);
        assert!(save_generated(dir.path(), &other, &bad).is_err());
        assert!(local(dir.path(), &other).is_none());
    }
    #[test]
    fn replacing_audio_invalidates_cache_identity() {
        let dir = tempfile::tempdir().unwrap();
        let mut q = query();
        q.path = dir.path().join("audio.wav").to_string_lossy().into();
        std::fs::write(&q.path, b"first").unwrap();
        let original = key(&q);
        std::fs::write(&q.path, b"new audio data").unwrap();
        assert_ne!(key(&q), original);
    }
    #[test]
    fn weak_or_non_speech_segments_are_not_saved_as_lyrics() {
        for metric in ["no_speech_prob", "avg_logprob", "compression_ratio"] {
            let mut segment = serde_json::json!({"start":1,"end":2,"text":"Unreliable"});
            segment[metric] = serde_json::json!(match metric {
                "no_speech_prob" => 0.99,
                "avg_logprob" => -2.0,
                _ => 3.0,
            });
            assert!(whisper_to_lrc(&serde_json::json!({"segments":[segment]}), 10.0).is_err());
        }
    }
    #[test]
    fn top_level_words_and_timestamps_are_validated() {
        let v = serde_json::json!({"segments":[{"start":1,"end":3,"text":"Xin chào"}],"words":[{"start":1,"end":1.5,"word":"Xin"},{"start":2,"end":3,"word":"chào"}]});
        let lrc = whisper_to_lrc(&v, 10.0).unwrap();
        assert!(lrc.contains("<00:01.00>Xin"));
        assert!(lrc.contains("<00:03.00>"));
        assert_eq!(stamp(59.999).unwrap(), "01:00.00");
        let mut bad = v.clone();
        bad["segments"][0]["start"] = (-1).into();
        assert!(whisper_to_lrc(&bad, 10.0).is_err());
        assert!(whisper_to_lrc(&serde_json::json!({"text":"invented"}), 10.0).is_err());
    }
    #[test]
    fn word_timing_keeps_segment_spelling_and_each_repetition() {
        let v = serde_json::json!({"segments":[{"start":1,"end":4,"text":"La la, la!","words":[
            {"start":1,"end":1.1,"word":"la"},
            {"start":1.1,"end":1.3,"word":"la"},
            {"start":2,"end":4,"word":"la"}
        ]}]});
        assert_eq!(
            whisper_to_lrc(&v, 5.0).unwrap(),
            "[00:01.00]<00:01.00>La <00:01.10>la, <00:01.30> <00:02.00>la!<00:04.00>"
        );
        assert!(whisper_to_lrc(&v, f64::NAN).is_err());
        let mut zero = v.clone();
        zero["segments"][0]["words"][0]["end"] = serde_json::json!(1);
        assert_eq!(whisper_to_lrc(&zero, 5.0).unwrap(), "[00:01.00]La la, la!");
        let mut wrong = v.clone();
        wrong["segments"][0]["words"][2]["word"] = serde_json::json!("khác");
        assert_eq!(whisper_to_lrc(&wrong, 5.0).unwrap(), "[00:01.00]La la, la!");
    }
    #[test]
    fn captions_keep_later_repeats_append_events_and_word_offsets() {
        let v = serde_json::json!({"events":[
            {"tStartMs":1000,"dDurationMs":2000,"segs":[{"utf8":"La"},{"utf8":" la","tOffsetMs":100},{"utf8":" la","tOffsetMs":500}]},
            {"tStartMs":4000,"dDurationMs":1000,"segs":[{"utf8":"La la la"}]},
            {"tStartMs":4500,"dDurationMs":1000,"aAppend":1,"segs":[{"utf8":"La la la"}]},
            {"tStartMs":4600,"dDurationMs":1000,"segs":[{"utf8":"La la la"}]}
        ]});
        assert_eq!(
            captions_to_lrc(&v, 10.0).unwrap(),
            "[00:01.00]<00:01.00>La <00:01.10>la <00:01.50>la<00:03.00>\n[00:04.00]La la la\n[00:04.50]La la la\n[00:04.60]La la la"
        );
        let unknown = serde_json::json!({"events":[
            {"tStartMs":1000,"segs":[{"utf8":"La"}]},
            {"tStartMs":1100,"segs":[{"utf8":"La"}]}
        ]});
        assert_eq!(
            captions_to_lrc(&unknown, 5.0).unwrap(),
            "[00:01.00]La\n[00:01.10]La"
        );
    }
    #[test]
    fn rolling_captions_do_not_duplicate_words() {
        let v = serde_json::json!({"events":[{"tStartMs":1000,"dDurationMs":2000,"segs":[{"utf8":"Xin chào"}]},{"tStartMs":1500,"dDurationMs":2000,"segs":[{"utf8":"Xin chào Việt Nam"}]},{"tStartMs":1500,"dDurationMs":2000,"segs":[{"utf8":"Xin chào Việt Nam"}]}]});
        assert_eq!(
            captions_to_lrc(&v, 10.0).unwrap(),
            "[00:01.00]Xin chào\n[00:01.50]Việt Nam"
        );
    }
}
