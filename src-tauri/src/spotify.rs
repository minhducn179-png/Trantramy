use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

fn spotify_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(4))
            .timeout_read(Duration::from_secs(8))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .build()
    })
}

fn spotify_cache() -> &'static Mutex<HashMap<String, SpotifyCollection>> {
    static CACHE: OnceLock<Mutex<HashMap<String, SpotifyCollection>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SpotifyKind {
    Playlist,
    Album,
    Track,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrackItem {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: u64, // seconds
    pub thumbnail: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCollection {
    pub kind: SpotifyKind,
    pub id: String,
    pub title: String,
    pub author: String,
    pub thumbnail: Option<String>,
    pub total: usize,
    pub tracks: Vec<SpotifyTrackItem>,
}

pub fn parse_spotify_input(input: &str) -> Option<(SpotifyKind, String)> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }

    // URI format: spotify:playlist:xxxx or spotify:album:xxxx or spotify:track:xxxx
    if let Some(rest) = clean.strip_prefix("spotify:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() >= 2 {
            let kind = match parts[0] {
                "playlist" => SpotifyKind::Playlist,
                "album" => SpotifyKind::Album,
                "track" => SpotifyKind::Track,
                _ => return None,
            };
            let id = parts[1].split('?').next().unwrap_or(parts[1]).to_string();
            return Some((kind, id));
        }
    }

    // URL format: https://open.spotify.com/playlist/xxxx or /album/xxxx or /track/xxxx
    let url_without_query = clean.split('?').next().unwrap_or(clean);
    let parts: Vec<&str> = url_without_query.trim_end_matches('/').split('/').collect();

    for i in 0..parts.len() {
        let seg = parts[i];
        let kind_opt = match seg {
            "playlist" => Some(SpotifyKind::Playlist),
            "album" => Some(SpotifyKind::Album),
            "track" => Some(SpotifyKind::Track),
            _ => None,
        };
        if let Some(kind) = kind_opt
            && i + 1 < parts.len()
        {
            let id = parts[i + 1].trim().to_string();
            if !id.is_empty() {
                return Some((kind, id));
            }
        }
    }

    None
}

/// Fetch Spotify data using the public embed page JSON (__NEXT_DATA__)
/// This is 100% reliable, fast, public, and requires NO API credentials or auth tokens!
pub fn fetch_spotify(
    input: &str,
    _client_id: Option<String>,
    _client_secret: Option<String>,
) -> Result<SpotifyCollection, String> {
    let (kind, id) = parse_spotify_input(input).ok_or_else(|| {
        "Đường dẫn Spotify không đúng định dạng (cần là playlist, album hoặc bài hát)".to_string()
    })?;

    let kind_str = match kind {
        SpotifyKind::Playlist => "playlist",
        SpotifyKind::Album => "album",
        SpotifyKind::Track => "track",
    };

    let cache_key = format!("{}:{}", kind_str, id);
    if let Some(cached) = spotify_cache().lock().unwrap().get(&cache_key).cloned() {
        return Ok(cached);
    }

    // 1. Fetch the embed HTML
    let embed_url = format!("https://open.spotify.com/embed/{}/{}", kind_str, id);
    let resp = spotify_agent()
        .get(&embed_url)
        .set(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .set("Accept-Language", "en-US,en;q=0.9,vi;q=0.8")
        .call();

    match resp {
        Ok(res) => {
            let html = res
                .into_string()
                .map_err(|e| format!("Lỗi đọc nội dung Spotify embed: {e}"))?;

            match parse_embed_html(&html, kind.clone(), id.clone()) {
                Ok(collection) => {
                    spotify_cache()
                        .lock()
                        .unwrap()
                        .insert(cache_key, collection.clone());
                    return Ok(collection);
                }
                Err(e) => {
                    // Try oEmbed fallback below
                    eprintln!("Embed parse error: {e}, falling back to oEmbed");
                }
            }
        }
        Err(e) => {
            eprintln!("Embed request error: {e}, falling back to oEmbed");
        }
    }

    // 2. Fallback to oEmbed if embed page was blocked or failed
    let col = fetch_oembed(input, kind, id)?;
    spotify_cache()
        .lock()
        .unwrap()
        .insert(cache_key, col.clone());
    Ok(col)
}

fn parse_embed_html(
    html: &str,
    kind: SpotifyKind,
    id: String,
) -> Result<SpotifyCollection, String> {
    let needle = "id=\"__NEXT_DATA__\"";
    let start_pos = html
        .find(needle)
        .ok_or_else(|| "Không tìm thấy dữ liệu __NEXT_DATA__ trong trang Spotify".to_string())?;

    let after_tag = &html[start_pos..];
    let content_start = after_tag
        .find('>')
        .ok_or_else(|| "Thẻ __NEXT_DATA__ không hợp lệ".to_string())?
        + 1;
    let content_slice = &after_tag[content_start..];

    let end_pos = content_slice
        .find("</script>")
        .ok_or_else(|| "Không tìm thấy thẻ đóng script trong trang Spotify".to_string())?;

    let json_str = &content_slice[..end_pos].trim();
    let data: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Lỗi phân giải JSON Spotify: {e}"))?;

    let entity = &data["props"]["pageProps"]["state"]["data"]["entity"];
    if entity.is_null() {
        return Err("Dữ liệu Spotify entity trống".to_string());
    }

    let title = entity["name"]
        .as_str()
        .or_else(|| entity["title"].as_str())
        .unwrap_or("Spotify Collection")
        .to_string();

    let author = entity["subtitle"]
        .as_str()
        .or_else(|| entity["owner"]["name"].as_str())
        .or_else(|| {
            entity["artists"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|a| a["name"].as_str())
        })
        .unwrap_or("Spotify")
        .to_string();

    let thumbnail = entity["visualIdentity"]["image"]
        .as_array()
        .and_then(|arr| arr.get(1).or_else(|| arr.first()))
        .and_then(|img| img["url"].as_str())
        .map(|s| s.to_string());

    let mut tracks = Vec::new();

    // Check if it's a playlist or album with trackList
    if let Some(list) = entity["trackList"].as_array() {
        for item in list {
            let t_name = item["title"].as_str().unwrap_or("").to_string();
            if t_name.is_empty() {
                continue;
            }
            let artist = item["subtitle"].as_str().unwrap_or(&author).to_string();
            let duration_ms = item["duration"].as_u64().unwrap_or(0);
            let uri = item["uri"].as_str().unwrap_or("");
            let clean_track_id = uri.replace("spotify:track:", "");
            let track_url = if !clean_track_id.is_empty() {
                format!("https://open.spotify.com/track/{}", clean_track_id)
            } else {
                format!("https://open.spotify.com/search/{}", t_name)
            };

            tracks.push(SpotifyTrackItem {
                title: t_name,
                artist,
                album: Some(title.clone()),
                duration: duration_ms / 1000,
                thumbnail: thumbnail.clone(),
                url: track_url,
            });
        }
    } else if kind == SpotifyKind::Track {
        // Single track embed
        let duration_ms = entity["duration"].as_u64().unwrap_or(0);
        tracks.push(SpotifyTrackItem {
            title: title.clone(),
            artist: author.clone(),
            album: None,
            duration: duration_ms / 1000,
            thumbnail: thumbnail.clone(),
            url: format!("https://open.spotify.com/track/{}", id),
        });
    }

    let total = tracks.len();
    Ok(SpotifyCollection {
        kind,
        id,
        title,
        author,
        thumbnail,
        total,
        tracks,
    })
}

fn fetch_oembed(input: &str, kind: SpotifyKind, id: String) -> Result<SpotifyCollection, String> {
    let oembed_url = format!(
        "https://open.spotify.com/oembed?url={}",
        url_encode(input.trim())
    );
    let resp = ureq::get(&oembed_url)
        .set("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| format!("Không thể lấy thông tin Spotify (oEmbed): {e}"))?;

    let val: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Lỗi phân giải JSON oEmbed: {e}"))?;

    let title = val["title"]
        .as_str()
        .unwrap_or("Spotify Collection")
        .to_string();
    let author = val["provider_name"]
        .as_str()
        .unwrap_or("Spotify")
        .to_string();
    let thumbnail = val["thumbnail_url"].as_str().map(|s| s.to_string());

    let single_track = SpotifyTrackItem {
        title: title.clone(),
        artist: author.clone(),
        album: None,
        duration: 0,
        thumbnail: thumbnail.clone(),
        url: input.trim().to_string(),
    };

    Ok(SpotifyCollection {
        kind,
        id,
        title,
        author,
        thumbnail,
        total: 1,
        tracks: vec![single_track],
    })
}

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spotify_input() {
        assert_eq!(
            parse_spotify_input(
                "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=12345"
            ),
            Some((SpotifyKind::Playlist, "37i9dQZF1DXcBWIGoYBM5M".to_string()))
        );
        assert_eq!(
            parse_spotify_input("spotify:album:4m2880jivSbbyEGAKfITCa"),
            Some((SpotifyKind::Album, "4m2880jivSbbyEGAKfITCa".to_string()))
        );
        assert_eq!(
            parse_spotify_input("https://open.spotify.com/track/5ViLKrbyL3HD6wsq3AB9eI"),
            Some((SpotifyKind::Track, "5ViLKrbyL3HD6wsq3AB9eI".to_string()))
        );
    }
}
