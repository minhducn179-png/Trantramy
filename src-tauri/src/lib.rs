mod db;
mod lyrics;
mod lyrics_ai;
mod lyrics_jobs;
mod lyrics_pipeline;
mod lyrics_review;
mod recommend;
mod scanner;
mod spotify;
mod ytdlp;

use db::{Db, Playlist, Track};
use lyrics_jobs::{lyrics_delete, lyrics_review_text, lyrics_transcribe, LyricsService, lyrics_read, lyrics_store_key, lyrics_settings, lyrics_save_settings, lyrics_download_model, lyrics_cancel_download, lyrics_generate, lyrics_queue, lyrics_pause, lyrics_start_library, open_lyrics_folder};
use recommend::RecCache;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};
use ytdlp::{OnlineTrack, SearchCache, StreamCache};

fn err<E: ToString>(e: E) -> String {
    e.to_string()
}

#[derive(serde::Serialize, Clone)]
struct ScanProgress {
    done: usize,
    total: usize,
}

#[tauri::command]
async fn scan_folder(app: AppHandle, path: String) -> Result<usize, String> {
    let cover_dir: PathBuf = app.path().app_cache_dir().map_err(err)?.join("covers");
    let files = tauri::async_runtime::spawn_blocking({
        let p = PathBuf::from(&path);
        move || scanner::collect_files(&p)
    })
    .await
    .map_err(err)?;
    let total = files.len();
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let db = handle.state::<Db>();
        let mut ok = 0;
        for (i, f) in files.iter().enumerate() {
            {
                let conn = db.0.lock().unwrap();
                if scanner::index_file(&conn, f, &cover_dir).is_ok() {
                    ok += 1;
                }
            }
            if i % 10 == 0 || i + 1 == total {
                let _ = handle.emit("scan-progress", ScanProgress { done: i + 1, total });
            }
        }
        ok
    })
    .await
    .map_err(err)
}

#[tauri::command]
fn list_tracks(db: State<Db>, query: Option<String>) -> Result<Vec<Track>, String> {
    db::list_tracks(&db.0.lock().unwrap(), &query.unwrap_or_default()).map_err(err)
}

#[tauri::command]
fn delete_track(db: State<Db>, id: i64) -> Result<(), String> {
    db::delete_track(&db.0.lock().unwrap(), id).map_err(err)
}

#[tauri::command]
fn list_playlists(db: State<Db>) -> Result<Vec<Playlist>, String> {
    db::list_playlists(&db.0.lock().unwrap()).map_err(err)
}

#[tauri::command]
async fn get_lyrics(
    app: AppHandle,
    path: String,
    title: String,
    artist: String,
    album: String,
    duration: f64,
    url: Option<String>,
) -> Result<Option<String>, String> {
    let q = lyrics::LyricsQuery { path, title, artist, album, duration, url };
    tauri::async_runtime::spawn_blocking(move || {
        let automatic = app.state::<LyricsService>().config.lock().map_err(|_| "Cấu hình lyrics đang lỗi.")?.automatic;
        lyrics_jobs::generate(&app, &q, automatic, &std::sync::atomic::AtomicBool::new(false), false)
            .map(|r| r.filter(|r| !r.draft_only).map(|r|r.content))
    }).await.map_err(err)?
}

#[tauri::command]
async fn save_lyrics(
    app: AppHandle,
    path: String,
    title: String,
    artist: String,
    album: String,
    duration: f64,
    content: String,
) -> Result<(), String> {
    let q = lyrics::LyricsQuery { path, title, artist, album, duration, url: None };
    tauri::async_runtime::spawn_blocking(move || {
        lyrics_jobs::valid_query(&q)?;
        let service = app.state::<LyricsService>();
        let _gate = service.gate.try_lock().map_err(|_| "Đang có tác vụ lời khác. Chờ xong hoặc tạm dừng hàng đợi rồi lưu.")?;
        lyrics::save_manual(&service.cache_dir(), &q, &content)?;
        let _ = app.emit("lyrics-updated", &q.path);
        Ok(())
    }).await.map_err(err)?
}

#[tauri::command]
fn rename_playlist(db: State<Db>, id: i64, name: String) -> Result<(), String> {
    db::rename_playlist(&db.0.lock().unwrap(), id, &name).map_err(err)
}

/// Copies the chosen image into the app data dir (so it survives the original being moved)
/// and stores it as the playlist logo. `src = None` removes the logo.
#[tauri::command]
fn set_playlist_cover(
    app: AppHandle,
    db: State<Db>,
    id: i64,
    src: Option<String>,
) -> Result<Option<String>, String> {
    let dest = match src {
        Some(s) => {
            if s.starts_with("http://") || s.starts_with("https://") {
                let dir = app.path().app_data_dir().map_err(err)?.join("playlist_covers");
                Some(ytdlp::cache_thumbnail(&dir, &s).unwrap_or(s))
            } else {
                let src_path = PathBuf::from(&s);
                let ext = src_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .filter(|e| ["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&e.as_str()))
                    .ok_or("Định dạng ảnh không được hỗ trợ")?;
                let dir = app.path().app_data_dir().map_err(err)?.join("playlist_covers");
                std::fs::create_dir_all(&dir).map_err(err)?;
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                let file = dir.join(format!("{}_{}.{}", id, ts, ext));
                std::fs::copy(&src_path, &file).map_err(err)?;
                Some(file.to_string_lossy().to_string())
            }
        }
        None => None,
    };
    db::set_playlist_cover(&db.0.lock().unwrap(), id, dest.as_deref()).map_err(err)?;
    Ok(dest)
}

#[tauri::command]
fn save_background_image(app: AppHandle, src: String) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(err)?.join("backgrounds");
    std::fs::create_dir_all(&dir).map_err(err)?;
    if src.starts_with("http://") || src.starts_with("https://") {
        Ok(ytdlp::cache_thumbnail(&dir, &src).unwrap_or(src))
    } else {
        let src_path = PathBuf::from(&src);
        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .filter(|e| {
                ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg"].contains(&e.as_str())
            })
            .ok_or("Định dạng ảnh phông nền không được hỗ trợ")?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let file = dir.join(format!("bg_{}.{}", ts, ext));
        std::fs::copy(&src_path, &file).map_err(err)?;
        Ok(file.to_string_lossy().to_string())
    }
}

#[tauri::command]
fn save_background_bytes(app: AppHandle, ext: String, data: Vec<u8>) -> Result<String, String> {
    if data.is_empty() {
        return Err("Dữ liệu ảnh trống".into());
    }
    let clean_ext = ext
        .trim()
        .trim_start_matches('.')
        .to_lowercase();
    let valid_ext = if ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg"]
        .contains(&clean_ext.as_str())
    {
        clean_ext
    } else {
        "png".to_string()
    };
    let dir = app.path().app_data_dir().map_err(err)?.join("backgrounds");
    std::fs::create_dir_all(&dir).map_err(err)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file = dir.join(format!("bg_{}.{}", ts, valid_ext));
    std::fs::write(&file, &data).map_err(err)?;
    Ok(file.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_background_image(app: AppHandle, path: String) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(err)?.join("backgrounds");
    let target = PathBuf::from(&path);
    if target.starts_with(&dir) && target.exists() {
        let _ = std::fs::remove_file(&target);
    }
    Ok(())
}

#[tauri::command]
fn create_playlist(db: State<Db>, name: String) -> Result<i64, String> {
    db::create_playlist(&db.0.lock().unwrap(), &name).map_err(err)
}

#[tauri::command]
fn delete_playlist(db: State<Db>, id: i64) -> Result<(), String> {
    db::delete_playlist(&db.0.lock().unwrap(), id).map_err(err)
}

#[tauri::command]
fn add_to_playlist(db: State<Db>, playlist_id: i64, track_id: i64) -> Result<(), String> {
    db::add_to_playlist(&db.0.lock().unwrap(), playlist_id, track_id).map_err(err)
}

#[tauri::command]
fn remove_from_playlist(db: State<Db>, playlist_id: i64, track_id: i64) -> Result<(), String> {
    db::remove_from_playlist(&db.0.lock().unwrap(), playlist_id, track_id).map_err(err)
}

#[tauri::command]
fn get_playlist(db: State<Db>, id: i64) -> Result<Vec<Track>, String> {
    db::get_playlist(&db.0.lock().unwrap(), id).map_err(err)
}

#[tauri::command]
async fn search_online(
    app: AppHandle,
    query: String,
    source: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<OnlineTrack>, String> {
    let src = source.unwrap_or_else(|| "all".into());
    let lim = limit.unwrap_or(16).clamp(4, 60);
    ytdlp::search(&app, &query, &src, lim).await
}

#[tauri::command]
async fn resolve_info(app: AppHandle, url: String) -> Result<Vec<OnlineTrack>, String> {
    ytdlp::resolve_info(&app, &url).await
}

#[tauri::command]
async fn resolve_stream(
    app: AppHandle,
    cache: State<'_, StreamCache>,
    url: String,
) -> Result<String, String> {
    ytdlp::resolve_stream(&app, &cache, &url).await
}

#[tauri::command]
async fn add_online_to_library(app: AppHandle, track: OnlineTrack) -> Result<Track, String> {
    let cover_dir = app.path().app_cache_dir().map_err(err)?.join("covers");
    let thumb = track.thumbnail.clone();
    let local_cover = tauri::async_runtime::spawn_blocking(move || {
        thumb.as_deref().and_then(|u| ytdlp::cache_thumbnail(&cover_dir, u))
    })
    .await
    .map_err(err)?;

    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    let id = db::add_online_track(
        &conn,
        &track.url,
        (&track.title, &track.artist, &track.album),
        track.duration,
        local_cover.as_deref().or(track.thumbnail.as_deref()),
        &track.source,
    )
    .map_err(err)?;
    db::get_track(&conn, id)
        .map_err(err)?
        .ok_or_else(|| "Không tìm thấy bài vừa thêm".into())
}

#[tauri::command]
async fn download_track(
    app: AppHandle,
    track: OnlineTrack,
    track_id: Option<i64>,
) -> Result<Track, String> {
    let cover_dir = app.path().app_cache_dir().map_err(err)?.join("covers");
    let thumb = track.thumbnail.clone();
    let local_cover = tauri::async_runtime::spawn_blocking(move || {
        thumb.as_deref().and_then(|u| ytdlp::cache_thumbnail(&cover_dir, u))
    })
    .await
    .map_err(err)?;

    let id = {
        let db = app.state::<Db>();
        let conn = db.0.lock().unwrap();
        match track_id {
            Some(existing) => existing,
            None => db::add_online_track(
                &conn,
                &track.url,
                (&track.title, &track.artist, &track.album),
                track.duration,
                local_cover.as_deref().or(track.thumbnail.as_deref()),
                &track.source,
            )
            .map_err(err)?,
        }
    };

    let dest_dir = app
        .path()
        .audio_dir()
        .or_else(|_| app.path().app_data_dir())
        .map_err(err)?
        .join("dlc");

    let mut dl_url = track.url.clone();
    if track.source == "spotify" || dl_url.contains("spotify.com") {
        let q = format!("{} - {} official audio", track.artist, track.title);
        if let Ok(found) = ytdlp::search(&app, &q, "youtube", 1).await {
            if let Some(first) = found.first() {
                dl_url = first.url.clone();
            }
        }
    }

    let downloaded = ytdlp::download_audio(&app, &dl_url, &dest_dir, Some(id)).await?;
    let path_str = downloaded.to_string_lossy().to_string();

    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    db::set_track_file(&conn, id, &path_str, local_cover.as_deref()).map_err(err)?;
    db::get_track(&conn, id)
        .map_err(err)?
        .ok_or_else(|| "Không tìm thấy bài vừa tải".into())
}

#[tauri::command]
fn record_play(
    db: State<Db>,
    title: String,
    artist: String,
    source: String,
    url: Option<String>,
    completed: bool,
) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    db::record_play(
        &db.0.lock().unwrap(),
        &title,
        &artist,
        &source,
        url.as_deref(),
        now,
        completed,
    )
    .map_err(err)
}

#[tauri::command]
fn toggle_like(db: State<Db>, id: i64) -> Result<bool, String> {
    db::toggle_like_track(&db.0.lock().unwrap(), id).map_err(err)
}

#[tauri::command]
fn list_liked(db: State<Db>) -> Result<Vec<Track>, String> {
    db::list_liked_tracks(&db.0.lock().unwrap()).map_err(err)
}

#[derive(serde::Serialize)]
struct HomeData {
    recommended: Vec<OnlineTrack>,
    library_recent: Vec<Track>,
    liked_tracks: Vec<Track>,
    history_recent: Vec<db::HistoryRow>,
    top_artists: Vec<(String, i64)>,
}

#[tauri::command]
async fn get_home_data(app: AppHandle, force: Option<bool>) -> Result<HomeData, String> {
    let recs = recommend::get_recommendations(&app, force.unwrap_or(false))
        .await
        .unwrap_or_default();
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    let library_recent = db::list_recent_library(&conn, 16).unwrap_or_default();
    let liked_tracks = db::list_liked_tracks(&conn).unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let history_recent = db::recent_history(&conn, now - 30 * 86400).unwrap_or_default();
    let top_artists = db::top_library_artists(&conn, 8).unwrap_or_default();

    Ok(HomeData {
        recommended: recs,
        library_recent,
        liked_tracks,
        history_recent,
        top_artists,
    })
}

#[tauri::command]
async fn get_recommendations(
    app: AppHandle,
    force: Option<bool>,
) -> Result<Vec<OnlineTrack>, String> {
    recommend::get_recommendations(&app, force.unwrap_or(false)).await
}

#[tauri::command]
fn window_action(app: AppHandle, action: String) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        match action.as_str() {
            "minimize" => {
                let _ = w.minimize();
            }
            "maximize" => {
                if w.is_maximized().unwrap_or(false) {
                    let _ = w.unmaximize();
                } else {
                    let _ = w.maximize();
                }
            }
            "close" => {
                app.exit(0);
            }
            "drag" => {
                let _ = w.start_dragging();
            }
            _ => {}
        }
    }
    Ok(())
}

#[tauri::command]
fn get_audio_proxy_url(url: String) -> String {
    url
}

#[cfg(target_os = "windows")]
mod win_icon {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn BeginUpdateResourceW(pFileName: *const u16, bDeleteExistingResources: i32) -> *mut c_void;
        fn UpdateResourceW(
            hUpdate: *mut c_void,
            lpType: *const u16,
            lpName: *const u16,
            wLanguage: u16,
            lpData: *const c_void,
            cb: u32,
        ) -> i32;
        fn EndUpdateResourceW(hUpdate: *mut c_void, fDiscard: i32) -> i32;
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHChangeNotify(wEventId: i32, uFlags: u32, dwItem1: *const c_void, dwItem2: *const c_void);
    }

    fn to_wide(p: &Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    fn resize_rgba(src: &[u8], sw: u32, sh: u32, dst: u32) -> Vec<u8> {
        let mut out = vec![0u8; (dst * dst * 4) as usize];
        for y in 0..dst {
            let sy = ((y as f32 + 0.5) * (sh as f32) / (dst as f32) - 0.5).clamp(0.0, (sh - 1) as f32);
            let y0 = sy.floor() as u32;
            let y1 = (y0 + 1).min(sh - 1);
            let fy = sy - y0 as f32;
            for x in 0..dst {
                let sx = ((x as f32 + 0.5) * (sw as f32) / (dst as f32) - 0.5).clamp(0.0, (sw - 1) as f32);
                let x0 = sx.floor() as u32;
                let x1 = (x0 + 1).min(sw - 1);
                let fx = sx - x0 as f32;

                let idx00 = ((y0 * sw + x0) * 4) as usize;
                let idx10 = ((y0 * sw + x1) * 4) as usize;
                let idx01 = ((y1 * sw + x0) * 4) as usize;
                let idx11 = ((y1 * sw + x1) * 4) as usize;
                let out_idx = ((y * dst + x) * 4) as usize;

                for c in 0..4 {
                    let v00 = src[idx00 + c] as f32;
                    let v10 = src[idx10 + c] as f32;
                    let v01 = src[idx01 + c] as f32;
                    let v11 = src[idx11 + c] as f32;
                    let top = v00 * (1.0 - fx) + v10 * fx;
                    let bot = v01 * (1.0 - fx) + v11 * fx;
                    out[out_idx + c] = (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        out
    }

    fn encode_dib_frame(rgba: &[u8], s: u32) -> Vec<u8> {
        let mask_row_bytes = ((s + 31) / 32) * 4;
        let xor_bytes = s * s * 4;
        let and_bytes = mask_row_bytes * s;
        let img_size = xor_bytes + and_bytes;
        let mut buf = Vec::with_capacity((40 + img_size) as usize);

        // BITMAPINFOHEADER (40 bytes)
        buf.extend_from_slice(&40u32.to_le_bytes());
        buf.extend_from_slice(&(s as i32).to_le_bytes());
        buf.extend_from_slice(&((s * 2) as i32).to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&32u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&img_size.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        // Bottom-up BGRA pixels
        for y in (0..s).rev() {
            for x in 0..s {
                let i = ((y * s + x) * 4) as usize;
                let r = rgba[i];
                let g = rgba[i + 1];
                let b = rgba[i + 2];
                let a = rgba[i + 3];
                if a == 0 {
                    buf.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    buf.extend_from_slice(&[b, g, r, a]);
                }
            }
        }

        // Bottom-up 1-bit AND mask
        for y in (0..s).rev() {
            let mut row = vec![0u8; mask_row_bytes as usize];
            for x in 0..s {
                let a = rgba[((y * s + x) * 4 + 3) as usize];
                if a < 16 {
                    row[(x / 8) as usize] |= 1 << (7 - (x % 8));
                }
            }
            buf.extend_from_slice(&row);
        }

        buf
    }

    pub fn sync_exe_and_ico_files(rgba: &[u8], width: u32, height: u32) {
        let sizes = [16u32, 24, 32, 48, 64, 256];
        let mut frames = Vec::with_capacity(sizes.len());
        for &s in &sizes {
            let scaled = if width == s && height == s {
                rgba.to_vec()
            } else {
                resize_rgba(rgba, width, height, s)
            };
            frames.push((s, encode_dib_frame(&scaled, s)));
        }

        // Build .ico file bytes
        let count = frames.len() as u16;
        let mut ico_bytes = Vec::new();
        ico_bytes.extend_from_slice(&0u16.to_le_bytes());
        ico_bytes.extend_from_slice(&1u16.to_le_bytes());
        ico_bytes.extend_from_slice(&count.to_le_bytes());

        let mut offset = 6u32 + (count as u32) * 16;
        let mut grp_dir = Vec::with_capacity(6 + frames.len() * 14);
        grp_dir.extend_from_slice(&0u16.to_le_bytes());
        grp_dir.extend_from_slice(&1u16.to_le_bytes());
        grp_dir.extend_from_slice(&count.to_le_bytes());

        for (idx, (s, data)) in frames.iter().enumerate() {
            let w_byte = if *s >= 256 { 0u8 } else { *s as u8 };
            let h_byte = if *s >= 256 { 0u8 } else { *s as u8 };
            let len = data.len() as u32;
            let id = (idx + 1) as u16;

            // ICONDIRENTRY (16 bytes)
            ico_bytes.push(w_byte);
            ico_bytes.push(h_byte);
            ico_bytes.push(0);
            ico_bytes.push(0);
            ico_bytes.extend_from_slice(&1u16.to_le_bytes());
            ico_bytes.extend_from_slice(&32u16.to_le_bytes());
            ico_bytes.extend_from_slice(&len.to_le_bytes());
            ico_bytes.extend_from_slice(&offset.to_le_bytes());
            offset += len;

            // GRPICONDIRENTRY (14 bytes)
            grp_dir.push(w_byte);
            grp_dir.push(h_byte);
            grp_dir.push(0);
            grp_dir.push(0);
            grp_dir.extend_from_slice(&1u16.to_le_bytes());
            grp_dir.extend_from_slice(&32u16.to_le_bytes());
            grp_dir.extend_from_slice(&len.to_le_bytes());
            grp_dir.extend_from_slice(&id.to_le_bytes());
        }

        for (_, data) in &frames {
            ico_bytes.extend_from_slice(data);
        }

        let ico_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("icons").join("icon.ico")))
            .unwrap_or_else(|| std::path::PathBuf::from("icons/icon.ico"));
        if let Some(parent) = ico_path.parent() {
            if parent.exists() {
                let _ = std::fs::write(ico_path, &ico_bytes);
            }
        }

        // Update embedded RT_ICON / RT_GROUP_ICON inside c:\miu dich\dlc.exe
        static SYNC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = match SYNC_LOCK.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let exe_path = std::env::current_exe().unwrap_or_default();
        if !exe_path.exists() {
            return;
        }

        let wide_exe = to_wide(&exe_path);
        unsafe {
            let mut h = BeginUpdateResourceW(wide_exe.as_ptr(), 0);
            if h.is_null() {
                // First time while this process is mapped to c:\miu dich\dlc.exe:
                // Move the mapped binary to target\dlc_running_<pid>.exe on the same volume and copy back an unlocked file.
                let target_dir = exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::env::temp_dir());
                let temp_old = target_dir.join(format!("dlc_running_{}.exe", std::process::id()));
                let _ = std::fs::remove_file(&temp_old);
                if std::fs::rename(&exe_path, &temp_old).is_ok() {
                    if std::fs::copy(&temp_old, &exe_path).is_ok() {
                        h = BeginUpdateResourceW(wide_exe.as_ptr(), 0);
                    } else {
                        let _ = std::fs::rename(&temp_old, &exe_path);
                    }
                }
            }

            if !h.is_null() {
                // RT_ICON = 3, RT_GROUP_ICON = 14, lang = 1033
                for (idx, (_, data)) in frames.iter().enumerate() {
                    let id = (idx + 1) as usize;
                    let _ = UpdateResourceW(
                        h,
                        3 as *const u16,
                        id as *const u16,
                        1033,
                        data.as_ptr() as *const c_void,
                        data.len() as u32,
                    );
                }
                let _ = UpdateResourceW(
                    h,
                    14 as *const u16,
                    32512 as *const u16,
                    1033,
                    grp_dir.as_ptr() as *const c_void,
                    grp_dir.len() as u32,
                );
                let _ = EndUpdateResourceW(h, 0);

                // Notify Windows Explorer to refresh file icon immediately
                SHChangeNotify(0x00002000, 0x0005 | 0x1000, wide_exe.as_ptr() as *const c_void, std::ptr::null());
                SHChangeNotify(0x08000000, 0x0000 | 0x1000, std::ptr::null(), std::ptr::null());
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("ie4uinit.exe")
                    .arg("-ClearIconCache")
                    .creation_flags(0x08000000)
                    .spawn();
                let _ = std::process::Command::new("ie4uinit.exe")
                    .arg("-show")
                    .creation_flags(0x08000000)
                    .spawn();
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod win_stealth {
    use std::ffi::c_void;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowDisplayAffinity(hWnd: *mut c_void, dwAffinity: u32) -> i32;
        fn EnumWindows(
            lpEnumFunc: unsafe extern "system" fn(*mut c_void, isize) -> i32,
            lParam: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(hWnd: *mut c_void, lpdwProcessId: *mut u32) -> u32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetProcessMitigationPolicy(
            MitigationPolicy: i32,
            lpBuffer: *const c_void,
            dwLength: usize,
        ) -> i32;
    }

    const WDA_NONE: u32 = 0x00000000;
    const WDA_MONITOR: u32 = 0x00000001;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    const PROCESS_EXTENSION_POINT_DISABLE_POLICY: i32 = 7;

    pub fn disable_third_party_overlay_hooks() {
        unsafe {
            // Bit 0 = DisableExtensionPoints (blocks SetWindowsHookEx / AppInit_DLLs injection from Overwolf/Outplayed)
            let policy_flags: u32 = 1;
            let _ = SetProcessMitigationPolicy(
                PROCESS_EXTENSION_POINT_DISABLE_POLICY,
                &policy_flags as *const u32 as *const c_void,
                std::mem::size_of::<u32>(),
            );
        }
    }

    unsafe extern "system" fn enum_proc(hwnd: *mut c_void, lparam: isize) -> i32 {
        let mut pid: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == std::process::id() {
                let enable = lparam != 0;
                if enable {
                    if SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) == 0 {
                        let _ = SetWindowDisplayAffinity(hwnd, WDA_MONITOR);
                    }
                } else {
                    let _ = SetWindowDisplayAffinity(hwnd, WDA_NONE);
                }
            }
        }
        1
    }

    pub fn apply_to_process_windows(enable: bool) {
        unsafe {
            let _ = EnumWindows(enum_proc, if enable { 1 } else { 0 });
        }
    }
}

#[cfg(target_os = "windows")]
mod win_audio_spectrum {
    use std::ffi::c_void;
    use std::f32::consts::PI;
    use tauri::{AppHandle, Emitter};

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct WAVEFORMATEX {
        w_format_tag: u16,
        n_channels: u16,
        n_samples_per_sec: u32,
        n_avg_bytes_per_sec: u32,
        n_block_align: u16,
        w_bits_per_sample: u16,
        cb_size: u16,
    }

    #[repr(C)]
    struct IMMDeviceEnumeratorVtbl {
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        enum_audio_endpoints: *const c_void,
        get_default_audio_endpoint:
            unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void) -> i32,
    }

    #[repr(C)]
    struct IMMDeviceVtbl {
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        activate: unsafe extern "system" fn(
            *mut c_void,
            *const GUID,
            u32,
            *const c_void,
            *mut *mut c_void,
        ) -> i32,
    }

    #[repr(C)]
    struct IAudioClientVtbl {
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        initialize: unsafe extern "system" fn(
            *mut c_void,
            u32,
            u32,
            i64,
            i64,
            *const WAVEFORMATEX,
            *const GUID,
        ) -> i32,
        get_buffer_size: *const c_void,
        get_stream_latency: *const c_void,
        get_current_padding: *const c_void,
        is_format_supported: *const c_void,
        get_mix_format: unsafe extern "system" fn(*mut c_void, *mut *mut WAVEFORMATEX) -> i32,
        get_device_period: *const c_void,
        start: unsafe extern "system" fn(*mut c_void) -> i32,
        stop: unsafe extern "system" fn(*mut c_void) -> i32,
        reset: *const c_void,
        set_event_handle: *const c_void,
        get_service: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    }

    #[repr(C)]
    struct IAudioCaptureClientVtbl {
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        get_buffer: unsafe extern "system" fn(
            *mut c_void,
            *mut *mut u8,
            *mut u32,
            *mut u32,
            *mut u64,
            *mut u64,
        ) -> i32,
        release_buffer: unsafe extern "system" fn(*mut c_void, u32) -> i32,
        get_next_packet_size: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    }

    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoInitializeEx(pvReserved: *const c_void, dwCoInit: u32) -> i32;
        fn CoCreateInstance(
            rclsid: *const GUID,
            pUnkOuter: *const c_void,
            dwClsContext: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> i32;
    }

    const CLSID_MMDEVICE_ENUMERATOR: GUID = GUID {
        data1: 0xBCDE0395,
        data2: 0xE52F,
        data3: 0x467C,
        data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
    };
    const IID_IMMDEVICE_ENUMERATOR: GUID = GUID {
        data1: 0xA95664D2,
        data2: 0x9614,
        data3: 0x4F35,
        data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
    };
    const IID_IAUDIO_CLIENT: GUID = GUID {
        data1: 0x1CB9AD4C,
        data2: 0xDBFA,
        data3: 0x4C32,
        data4: [0xB1, 0x78, 0xC2, 0xF5, 0x68, 0xA7, 0x03, 0xB2],
    };
    const IID_IAUDIO_CAPTURE_CLIENT: GUID = GUID {
        data1: 0xC8ADBD64,
        data2: 0xE71E,
        data3: 0x48A0,
        data4: [0xA4, 0xDE, 0x18, 0x5C, 0x39, 0x5C, 0xD3, 0x17],
    };

    const CLSCTX_ALL: u32 = 23;
    const AUDCLNT_SHAREMODE_SHARED: u32 = 0;
    const AUDCLNT_STREAMFLAGS_LOOPBACK: u32 = 0x00020000;
    const AUDCLNT_BUFFERFLAGS_SILENT: u32 = 0x2;

    pub fn spawn_spectrum_listener(app: AppHandle) {
        std::thread::spawn(move || unsafe {
            let _ = CoInitializeEx(std::ptr::null(), 0);
            loop {
                run_capture_session(&app);
                std::thread::sleep(std::time::Duration::from_millis(1500));
            }
        });
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn run_capture_session(app: &AppHandle) {
        let mut enumerator: *mut c_void = std::ptr::null_mut();
        if CoCreateInstance(
            &CLSID_MMDEVICE_ENUMERATOR,
            std::ptr::null(),
            CLSCTX_ALL,
            &IID_IMMDEVICE_ENUMERATOR,
            &mut enumerator,
        ) < 0
            || enumerator.is_null()
        {
            return;
        }

        let enum_vtbl = *(enumerator as *const *const IMMDeviceEnumeratorVtbl);
        let mut device: *mut c_void = std::ptr::null_mut();
        // eRender = 0, eConsole = 0
        if ((*enum_vtbl).get_default_audio_endpoint)(enumerator, 0, 0, &mut device) < 0
            || device.is_null()
        {
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        let dev_vtbl = *(device as *const *const IMMDeviceVtbl);
        let mut audio_client: *mut c_void = std::ptr::null_mut();
        if ((*dev_vtbl).activate)(
            device,
            &IID_IAUDIO_CLIENT,
            CLSCTX_ALL,
            std::ptr::null(),
            &mut audio_client,
        ) < 0
            || audio_client.is_null()
        {
            ((*dev_vtbl).release)(device);
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        let client_vtbl = *(audio_client as *const *const IAudioClientVtbl);
        let mut pwfx: *mut WAVEFORMATEX = std::ptr::null_mut();
        if ((*client_vtbl).get_mix_format)(audio_client, &mut pwfx) < 0 || pwfx.is_null() {
            ((*client_vtbl).release)(audio_client);
            ((*dev_vtbl).release)(device);
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        let sample_rate = (*pwfx).n_samples_per_sec.max(8000) as f32;
        let channels = (*pwfx).n_channels.max(1) as usize;
        let bits = (*pwfx).w_bits_per_sample;
        let block_align = (*pwfx).n_block_align.max(1) as usize;

        if ((*client_vtbl).initialize)(
            audio_client,
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            500_000, // 50ms buffer
            0,
            pwfx,
            std::ptr::null(),
        ) < 0
        {
            ((*client_vtbl).release)(audio_client);
            ((*dev_vtbl).release)(device);
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        let mut capture_client: *mut c_void = std::ptr::null_mut();
        if ((*client_vtbl).get_service)(
            audio_client,
            &IID_IAUDIO_CAPTURE_CLIENT,
            &mut capture_client,
        ) < 0
            || capture_client.is_null()
        {
            ((*client_vtbl).release)(audio_client);
            ((*dev_vtbl).release)(device);
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        let cap_vtbl = *(capture_client as *const *const IAudioCaptureClientVtbl);
        if ((*client_vtbl).start)(audio_client) < 0 {
            ((*cap_vtbl).release)(capture_client);
            ((*client_vtbl).release)(audio_client);
            ((*dev_vtbl).release)(device);
            ((*enum_vtbl).release)(enumerator);
            return;
        }

        const N: usize = 512;
        let mut ring = [0.0f32; N];
        let mut ring_pos = 0usize;
        let mut smooth_bands = [0.0f32; 32];
        let mut zero_sent = false;

        // Precompute 32 logarithmic frequencies from 45 Hz to 11000 Hz
        let mut freqs = [0.0f32; 32];
        for i in 0..32 {
            let t = (i as f32) / 31.0;
            freqs[i] = 45.0 * (11000.0f32 / 45.0).powf(t);
        }

        let mut hann = [0.0f32; N];
        for n in 0..N {
            hann[n] = 0.5 * (1.0 - (2.0 * PI * (n as f32) / ((N - 1) as f32)).cos());
        }

        loop {
            std::thread::sleep(std::time::Duration::from_millis(28));

            let mut packet_len: u32 = 0;
            if ((*cap_vtbl).get_next_packet_size)(capture_client, &mut packet_len) < 0 {
                break;
            }

            let mut got_audio = false;
            while packet_len > 0 {
                let mut data_ptr: *mut u8 = std::ptr::null_mut();
                let mut num_frames: u32 = 0;
                let mut flags: u32 = 0;
                if ((*cap_vtbl).get_buffer)(
                    capture_client,
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) < 0
                {
                    break;
                }

                let frames = num_frames as usize;
                let silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT) != 0 || data_ptr.is_null();
                if !silent && frames > 0 {
                    got_audio = true;
                    for f in 0..frames {
                        let frame_ptr = data_ptr.add(f * block_align);
                        let mut mono = 0.0f32;
                        if bits == 32 {
                            let flt_ptr = frame_ptr as *const f32;
                            for c in 0..channels {
                                mono += *flt_ptr.add(c);
                            }
                        } else if bits == 16 {
                            let i16_ptr = frame_ptr as *const i16;
                            for c in 0..channels {
                                mono += (*i16_ptr.add(c) as f32) / 32768.0;
                            }
                        }
                        mono /= channels as f32;
                        ring[ring_pos] = mono;
                        ring_pos = (ring_pos + 1) % N;
                    }
                } else if frames > 0 {
                    for _ in 0..frames.min(N) {
                        ring[ring_pos] = 0.0;
                        ring_pos = (ring_pos + 1) % N;
                    }
                }

                let _ = ((*cap_vtbl).release_buffer)(capture_client, num_frames);
                if ((*cap_vtbl).get_next_packet_size)(capture_client, &mut packet_len) < 0 {
                    break;
                }
            }

            // Compute windowed samples & RMS energy
            let mut windowed = [0.0f32; N];
            let mut rms_sum = 0.0f32;
            for n in 0..N {
                let s = ring[(ring_pos + n) % N];
                rms_sum += s * s;
                windowed[n] = s * hann[n];
            }
            let rms = (rms_sum / (N as f32)).sqrt();

            let mut max_band_val = 0u8;
            let mut out_bytes = [0u8; 32];

            for b in 0..32 {
                let target_val = if got_audio && rms > 0.0004 {
                    let freq = freqs[b];
                    let omega = 2.0 * PI * freq / sample_rate;
                    let coeff = 2.0 * omega.cos();
                    let mut s1 = 0.0f32;
                    let mut s2 = 0.0f32;
                    for n in 0..N {
                        let s0 = windowed[n] + coeff * s1 - s2;
                        s2 = s1;
                        s1 = s0;
                    }
                    let power = (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0);
                    let mag = (power.sqrt() / (N as f32)) * (1.6 + (b as f32) * 0.24);
                    let norm = (mag * 28.0).powf(0.62).clamp(0.0, 1.0);
                    norm
                } else {
                    0.0
                };

                if target_val > smooth_bands[b] {
                    smooth_bands[b] = smooth_bands[b] * 0.32 + target_val * 0.68;
                } else {
                    smooth_bands[b] = (smooth_bands[b] * 0.72).max(0.0);
                }

                let byte_val = (smooth_bands[b] * 255.0).round().clamp(0.0, 255.0) as u8;
                out_bytes[b] = byte_val;
                if byte_val > max_band_val {
                    max_band_val = byte_val;
                }
            }

            if max_band_val > 2 {
                zero_sent = false;
                let _ = app.emit("dlc-audio-spectrum", out_bytes.to_vec());
            } else if !zero_sent {
                zero_sent = true;
                let _ = app.emit("dlc-audio-spectrum", vec![0u8; 32]);
            }
        }

        let _ = ((*client_vtbl).stop)(audio_client);
        ((*cap_vtbl).release)(capture_client);
        ((*client_vtbl).release)(audio_client);
        ((*dev_vtbl).release)(device);
        ((*enum_vtbl).release)(enumerator);
    }
}

#[tauri::command]
fn set_stealth_mode(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_content_protected(enabled);
    }
    if let Some(isl) = app.get_webview_window("island") {
        let _ = isl.set_content_protected(enabled);
    }
    #[cfg(target_os = "windows")]
    {
        if enabled {
            win_stealth::disable_third_party_overlay_hooks();
        }
        win_stealth::apply_to_process_windows(enabled);
    }
    Ok(())
}

#[tauri::command]
fn toggle_island_window(
    app: AppHandle,
    show: Option<bool>,
    stealth: Option<bool>,
) -> Result<bool, String> {
    let stealth_on = stealth.unwrap_or(true);
    if let Some(w) = app.get_webview_window("island") {
        let cur_vis = w.is_visible().unwrap_or(false);
        let next_vis = show.unwrap_or(!cur_vis);
        if next_vis {
            let _ = w.set_content_protected(stealth_on);
            let _ = w.show();
            let _ = w.set_always_on_top(true);
            #[cfg(target_os = "windows")]
            {
                win_stealth::apply_to_process_windows(stealth_on);
            }
        } else {
            let _ = w.hide();
        }
        let _ = app.emit("island-visibility-changed", next_vis);
        return Ok(next_vis);
    }

    if show == Some(false) {
        let _ = app.emit("island-visibility-changed", false);
        return Ok(false);
    }

    let w = tauri::WebviewWindowBuilder::new(
        &app,
        "island",
        tauri::WebviewUrl::App("/island".into()),
    )
    .title("DLC Dynamic Island")
    .inner_size(136.0, 40.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .focused(false)
    .content_protected(stealth_on)
    .build()
    .map_err(err)?;

    if let Ok(Some(mon)) = w.primary_monitor() {
        let scale = mon.scale_factor();
        let w_phys = (136.0 * scale).round() as i32;
        let x = mon.position().x + (mon.size().width as i32 - w_phys) / 2;
        let y = mon.position().y + (10.0 * scale).round() as i32;
        let _ = w.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
    }

    #[cfg(target_os = "windows")]
    {
        win_stealth::apply_to_process_windows(stealth_on);
    }

    let _ = app.emit("island-visibility-changed", true);
    Ok(true)
}

#[tauri::command]
fn set_island_expanded(app: AppHandle, expanded: bool) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("island") {
        let target_w = if expanded { 390.0 } else { 136.0 };
        let target_h = if expanded { 178.0 } else { 40.0 };
        let scale = w.scale_factor().unwrap_or(1.0);
        let new_w_phys = (target_w * scale).round() as u32;
        let new_h_phys = (target_h * scale).round() as u32;

        if let (Ok(pos), Ok(old_size)) = (w.outer_position(), w.outer_size()) {
            let center_x = pos.x + (old_size.width as i32) / 2;
            let new_x = center_x - (new_w_phys as i32) / 2;
            let _ = w.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                new_w_phys,
                new_h_phys,
            )));
            let _ = w.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                new_x, pos.y,
            )));
        } else {
            let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                target_w, target_h,
            )));
        }
        let _ = w.set_always_on_top(true);
    }
    Ok(())
}

#[tauri::command]
fn set_app_icon(app: AppHandle, rgba: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
    if rgba.len() != (width * height * 4) as usize {
        return Err("Kích thước dữ liệu biểu tượng không hợp lệ".into());
    }
    let img = tauri::image::Image::new(&rgba, width, height);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_icon(img.clone());
    }
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_icon(Some(img));
    }
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            win_icon::sync_exe_and_ico_files(&rgba, width, height);
        });
    }
    Ok(())
}

#[tauri::command]
async fn import_spotify(
    app: AppHandle,
    url: String,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<spotify::SpotifyCollection, String> {
    let col = tauri::async_runtime::spawn_blocking(move || {
        spotify::fetch_spotify(&url, client_id, client_secret)
    })
    .await
    .map_err(err)??;

    // Warm up the first track's YouTube search & stream in the background immediately
    if let Some(first) = col.tracks.first() {
        let q = format!("{} - {} official audio", first.artist, first.title);
        let app_bg = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = ytdlp::search(&app_bg, &q, "youtube", 1).await;
        });
    }

    Ok(col)
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut: &Shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let action = if shortcut.key == Code::MediaPlayPause {
                        "play-pause"
                    } else if shortcut.key == Code::MediaTrackNext {
                        "next"
                    } else if shortcut.key == Code::MediaTrackPrevious {
                        "prev"
                    } else {
                        return;
                    };
                    let _ = app.emit("media-key", action);
                })
                .build(),
        )
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("dlc.db");
            if !db_path.exists() {
                if let Some(parent) = dir.parent() {
                    let legacy_dir = parent.join("com.miu.player");
                    let legacy_db = legacy_dir.join("library.db");
                    if legacy_db.exists() {
                        let _ = std::fs::copy(&legacy_db, &db_path);
                    }
                    let ytdlp_name = if cfg!(target_os = "windows") { "yt-dlp.exe" } else { "yt-dlp" };
                    let legacy_ytdlp = legacy_dir.join(ytdlp_name);
                    let new_ytdlp = dir.join(ytdlp_name);
                    if legacy_ytdlp.exists() && !new_ytdlp.exists() {
                        let _ = std::fs::copy(&legacy_ytdlp, &new_ytdlp);
                    }
                }
            }
            let conn = db::open(&db_path)?;
            app.manage(Db(Mutex::new(conn)));
            app.manage(StreamCache::new());
            app.manage(SearchCache::new());
            app.manage(RecCache::new());
            app.manage(LyricsService::new(app.path().app_data_dir()?.join("lyrics"))?);

            // Enable anti-capture / anti-game-overlay stealth protection by default
            // (process-level hook blocking is applied by `set_stealth_mode` only while the user keeps Stealth Mode ON)
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_content_protected(true);
            }
            let _ = toggle_island_window(app.handle().clone(), Some(true), Some(true));
            #[cfg(target_os = "windows")]
            {
                win_stealth::apply_to_process_windows(true);
                win_audio_spectrum::spawn_spectrum_listener(app.handle().clone());
            }

            // Media keys (ignore failures, e.g. key already taken by another app)
            for code in [Code::MediaPlayPause, Code::MediaTrackNext, Code::MediaTrackPrevious] {
                let _ = app.global_shortcut().register(Shortcut::new(None, code));
            }

            // Tray
            let pp = MenuItem::with_id(app, "play-pause", "Phát / Tạm dừng", true, None::<&str>)?;
            let next = MenuItem::with_id(app, "next", "Bài tiếp theo", true, None::<&str>)?;
            let prev = MenuItem::with_id(app, "prev", "Bài trước", true, None::<&str>)?;
            let island_item = MenuItem::with_id(app, "toggle-island", "Bật / Tắt Dynamic Island", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Mở ứng dụng", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&pp, &prev, &next, &island_item, &show, &quit])?;
            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("DLC Audio Studio")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => show_main(app),
                    "toggle-island" => {
                        let _ = toggle_island_window(app.clone(), None, None);
                    }
                    id => {
                        let _ = app.emit("media-key", id);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            list_tracks,
            delete_track,
            list_playlists,
            create_playlist,
            rename_playlist,
            get_lyrics,
            save_lyrics,
            lyrics_read,
            lyrics_store_key,
            lyrics_settings,
            lyrics_save_settings,
            lyrics_download_model,
            lyrics_cancel_download,
            lyrics_generate,
            lyrics_delete,
            lyrics_review_text,
            lyrics_transcribe,
            lyrics_queue,
            lyrics_pause,
            lyrics_start_library,
            open_lyrics_folder,
            set_playlist_cover,
            save_background_image,
            save_background_bytes,
            delete_background_image,
            delete_playlist,
            add_to_playlist,
            remove_from_playlist,
            get_playlist,
            search_online,
            resolve_info,
            resolve_stream,
            add_online_to_library,
            download_track,
            record_play,
            get_recommendations,
            get_home_data,
            toggle_like,
            list_liked,
            window_action,
            get_audio_proxy_url,
            set_app_icon,
            set_stealth_mode,
            toggle_island_window,
            set_island_expanded,
            import_spotify
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
