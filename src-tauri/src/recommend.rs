use crate::db::{self, Db, HistoryRow};
use crate::ytdlp::{self, OnlineTrack};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

pub struct RecCache(pub Mutex<Option<(Vec<OnlineTrack>, Instant)>>);

impl RecCache {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn get(&self) -> Option<Vec<OnlineTrack>> {
        let guard = self.0.lock().unwrap();
        if let Some((items, ts)) = guard.as_ref() {
            if ts.elapsed() < Duration::from_secs(30 * 60) && !items.is_empty() {
                return Some(items.clone());
            }
        }
        None
    }

    pub fn put(&self, items: Vec<OnlineTrack>) {
        *self.0.lock().unwrap() = Some((items, Instant::now()));
    }
}

pub fn norm_key(title: &str, artist: &str) -> String {
    let clean = |s: &str| -> String {
        let low = s.to_lowercase();
        let stripped = low
            .replace("(official music video)", "")
            .replace("(official video)", "")
            .replace("(official audio)", "")
            .replace("[official video]", "")
            .replace("[official audio]", "")
            .replace("(lyrics)", "");
        stripped
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    format!("{}|{}", clean(artist), clean(title))
}

/// Scores artists using exponential time decay `0.97^(days_ago)` + completion bonus + library count fallback.
pub fn score_artists(
    history: &[HistoryRow],
    library_artists: &[(String, i64)],
    now: i64,
    limit: usize,
) -> Vec<String> {
    let mut scores: HashMap<String, (String, f64)> = HashMap::new();

    for h in history {
        let a = h.artist.trim();
        if a.is_empty() || a.eq_ignore_ascii_case("Unknown Artist") {
            continue;
        }
        let days = ((now - h.played_at).max(0) as f64) / 86_400.0;
        let bonus = if h.completed { 1.35 } else { 1.0 };
        let w = 0.97_f64.powf(days) * bonus;
        let key = a.to_lowercase();
        let entry = scores.entry(key).or_insert_with(|| (a.to_string(), 0.0));
        entry.1 += w;
    }

    for (artist, count) in library_artists {
        let a = artist.trim();
        if a.is_empty() || a.eq_ignore_ascii_case("Unknown Artist") {
            continue;
        }
        let key = a.to_lowercase();
        let entry = scores.entry(key).or_insert_with(|| (a.to_string(), 0.0));
        entry.1 += (*count as f64) * 0.45;
    }

    let mut ranked: Vec<(String, f64)> = scores.into_values().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.into_iter().take(limit).map(|(name, _)| name).collect()
}

pub fn dedupe_filter(
    candidates: Vec<OnlineTrack>,
    owned_keys: &HashSet<String>,
    owned_urls: &HashSet<String>,
) -> Vec<OnlineTrack> {
    let mut seen_keys = owned_keys.clone();
    let mut seen_urls = owned_urls.clone();
    let mut out = Vec::new();

    for c in candidates {
        if c.duration > 0.0 && (c.duration < 30.0 || c.duration > 1800.0) {
            continue;
        }
        let u = c.url.trim().to_lowercase();
        let k = norm_key(&c.title, &c.artist);
        if seen_urls.contains(&u) || seen_keys.contains(&k) {
            continue;
        }
        seen_urls.insert(u);
        seen_keys.insert(k);
        out.push(c);
    }
    out
}

fn extract_youtube_id(url: &str) -> Option<String> {
    if let Some(idx) = url.find("v=") {
        let rest = &url[idx + 2..];
        let id: String = rest.chars().take_while(|c| *c != '&' && *c != '#').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(idx) = url.find("youtu.be/") {
        let rest = &url[idx + 9..];
        let id: String = rest.chars().take_while(|c| *c != '?' && *c != '&').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

pub async fn get_recommendations(
    app: &AppHandle,
    force: bool,
) -> Result<Vec<OnlineTrack>, String> {
    let rec_cache = app.state::<RecCache>();
    let disk_cache = app.path().app_cache_dir().ok().map(|d| d.join("recs_cache.json"));
    if !force {
        if let Some(cached) = rec_cache.get() {
            return Ok(cached);
        }
        if let Some(ref path) = disk_cache {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(tracks) = serde_json::from_str::<Vec<OnlineTrack>>(&data) {
                    if !tracks.is_empty() {
                        rec_cache.put(tracks.clone());
                        return Ok(tracks);
                    }
                }
            }
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let since = now - 90 * 86_400;

    let (history, lib_artists, owned_tracks) = {
        let db = app.state::<Db>();
        let conn = db.0.lock().unwrap();
        let h = db::recent_history(&conn, since).map_err(|e| e.to_string())?;
        let la = db::top_library_artists(&conn, 8).map_err(|e| e.to_string())?;
        let tr = db::list_tracks(&conn, "").map_err(|e| e.to_string())?;
        (h, la, tr)
    };

    let mut owned_keys = HashSet::new();
    let mut owned_urls = HashSet::new();
    for t in &owned_tracks {
        owned_keys.insert(norm_key(&t.title, &t.artist));
        if let Some(u) = &t.url {
            owned_urls.insert(u.to_lowercase());
        }
        if t.path.starts_with("http") {
            owned_urls.insert(t.path.to_lowercase());
        }
    }
    // Avoid recommending tracks played in the last 15 entries
    for h in history.iter().take(15) {
        owned_keys.insert(norm_key(&h.title, &h.artist));
        if let Some(u) = &h.url {
            owned_urls.insert(u.to_lowercase());
        }
    }

    let top_artists = score_artists(&history, &lib_artists, now, 4);
    let mut buckets: Vec<Vec<OnlineTrack>> = Vec::new();

    // 1. YouTube Mix from most recent YouTube track in history or library
    let seed_yt = history
        .iter()
        .find_map(|h| h.url.as_deref().and_then(extract_youtube_id).map(|id| (id, h.title.clone())))
        .or_else(|| {
            owned_tracks
                .iter()
                .find_map(|t| t.url.as_deref().and_then(extract_youtube_id).map(|id| (id, t.title.clone())))
        });

    if let Some((vid, seed_title)) = seed_yt {
        let mix_url = format!("https://www.youtube.com/watch?v={vid}&list=RD{vid}");
        if let Ok(mut items) = ytdlp::resolve_info(app, &mix_url).await {
            for item in &mut items {
                item.reason = Some(format!("Mix từ “{seed_title}”"));
            }
            items.truncate(8);
            buckets.push(items);
        }
    }

    // 2. Queries per top artist (or starter topics when user has no history/library yet)
    let queries: Vec<(String, String)> = if top_artists.is_empty() {
        vec![
            ("Nhạc trẻ Việt chill mới nhất".into(), "Đề xuất mở đầu".into()),
            ("Indie Vietnam acoustic".into(), "Đề xuất mở đầu".into()),
            ("Lofi chill beats".into(), "Đề xuất mở đầu".into()),
        ]
    } else {
        top_artists
            .into_iter()
            .map(|a| (format!("{a} bài hát hay nhất"), format!("Vì bạn hay nghe {a}")))
            .collect()
    };

    for (q, reason) in queries {
        let app1 = app.clone();
        let q1 = q.clone();
        let app2 = app.clone();
        let q2 = q.clone();
        let h1 = tauri::async_runtime::spawn(async move {
            ytdlp::search_source(&app1, &q1, "youtube", 5).await
        });
        let h2 = tauri::async_runtime::spawn(async move {
            ytdlp::search_source(&app2, &q2, "soundcloud", 3).await
        });
        let yt = h1.await.ok().and_then(|r| r.ok()).unwrap_or_default();
        let sc = h2.await.ok().and_then(|r| r.ok()).unwrap_or_default();
        let mut group = Vec::new();
        for mut t in yt.into_iter().chain(sc) {
            t.reason = Some(reason.clone());
            group.push(t);
        }
        if !group.is_empty() {
            buckets.push(group);
        }
    }

    // Interleave across buckets for variety
    let mut interleaved = Vec::new();
    let max_len = buckets.iter().map(|b| b.len()).max().unwrap_or(0);
    for i in 0..max_len {
        for b in &buckets {
            if let Some(item) = b.get(i) {
                interleaved.push(item.clone());
            }
        }
    }

    let final_list = dedupe_filter(interleaved, &owned_keys, &owned_urls)
        .into_iter()
        .take(24)
        .collect::<Vec<_>>();

    rec_cache.put(final_list.clone());
    if let Some(ref path) = disk_cache {
        if let Ok(json) = serde_json::to_string(&final_list) {
            let _ = std::fs::write(path, json);
        }
    }
    Ok(final_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_recent_and_completed_higher() {
        let now = 1_000_000;
        let history = vec![
            HistoryRow {
                title: "A1".into(),
                artist: "Sơn Tùng M-TP".into(),
                source: "local".into(),
                url: None,
                played_at: now - 3600,
                completed: true,
            },
            HistoryRow {
                title: "A2".into(),
                artist: "Sơn Tùng M-TP".into(),
                source: "local".into(),
                url: None,
                played_at: now - 7200,
                completed: false,
            },
            HistoryRow {
                title: "B1".into(),
                artist: "Đen Vâu".into(),
                source: "local".into(),
                url: None,
                played_at: now - 60 * 86_400,
                completed: false,
            },
        ];
        let lib = vec![("Vũ.".to_string(), 3)];
        let top = score_artists(&history, &lib, now, 3);
        assert_eq!(top[0], "Sơn Tùng M-TP");
        assert!(top.contains(&"Vũ.".to_string()));
        assert!(top.contains(&"Đen Vâu".to_string()));
    }

    #[test]
    fn dedupes_and_filters_owned() {
        let mut owned_keys = HashSet::new();
        owned_keys.insert(norm_key("Lạc Trôi", "Sơn Tùng M-TP"));
        let owned_urls = HashSet::new();

        let candidates = vec![
            OnlineTrack {
                title: "Lạc Trôi (Official Video)".into(),
                artist: "Sơn Tùng M-TP".into(),
                album: "YT".into(),
                duration: 240.0,
                thumbnail: None,
                url: "https://youtube.com/watch?v=1".into(),
                source: "youtube".into(),
                reason: None,
            },
            OnlineTrack {
                title: "Bài Mới".into(),
                artist: "Ca Sĩ B".into(),
                album: "YT".into(),
                duration: 210.0,
                thumbnail: None,
                url: "https://youtube.com/watch?v=2".into(),
                source: "youtube".into(),
                reason: None,
            },
            OnlineTrack {
                title: "Bài Mới (Official Audio)".into(),
                artist: "Ca Sĩ B".into(),
                album: "SC".into(),
                duration: 210.0,
                thumbnail: None,
                url: "https://soundcloud.com/b/bai-moi".into(),
                source: "soundcloud".into(),
                reason: None,
            },
        ];

        let res = dedupe_filter(candidates, &owned_keys, &owned_urls);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].url, "https://youtube.com/watch?v=2");
    }
}
