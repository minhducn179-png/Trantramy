use crate::db;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

pub const EXTS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "opus", "aac"];

pub fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn collect_files(root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_audio(e.path()))
        .map(|e| e.into_path())
        .collect()
}

/// Reads tags of one file and stores it in the DB. Returns Err(String) on unreadable file.
pub fn index_file(conn: &Connection, path: &Path, cover_dir: &Path) -> Result<(), String> {
    let tagged = lofty::read_from_path(path).map_err(|e| e.to_string())?;
    let duration = tagged.properties().duration().as_secs_f64();
    let fallback = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let (mut title, mut artist, mut album, mut cover) = (
        fallback,
        "Unknown Artist".to_string(),
        "Unknown Album".to_string(),
        None,
    );

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        if let Some(t) = tag.title()
            && !t.trim().is_empty()
        {
            title = t.to_string();
        }
        if let Some(a) = tag.artist() {
            artist = a.to_string();
        }
        if let Some(a) = tag.album() {
            album = a.to_string();
        }
        if let Some(pic) = tag.pictures().first() {
            let hash = format!("{:x}", Sha256::digest(pic.data()));
            let ext = match pic.mime_type() {
                Some(lofty::picture::MimeType::Png) => "png",
                _ => "jpg",
            };
            let file = cover_dir.join(format!("{}.{}", &hash[..16], ext));
            if !file.exists() {
                let _ = std::fs::create_dir_all(cover_dir);
                let _ = std::fs::write(&file, pic.data());
            }
            cover = Some(file.to_string_lossy().to_string());
        }
    }

    db::upsert_track(
        conn,
        &path.to_string_lossy(),
        &title,
        &artist,
        &album,
        duration,
        cover.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_audio_files() {
        let dir = std::env::temp_dir().join("dlc_scan_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.MP3"), b"x").unwrap();
        std::fs::write(dir.join("sub").join("b.flac"), b"x").unwrap();
        std::fs::write(dir.join("c.txt"), b"x").unwrap();
        let files = collect_files(&dir);
        assert_eq!(files.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_file_returns_err_not_panic() {
        let dir = std::env::temp_dir().join("dlc_scan_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("bad.mp3");
        std::fs::write(&f, b"not audio").unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        let _ = index_file(&conn, &f, &dir.join("covers"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
