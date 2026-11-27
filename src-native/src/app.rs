use crate::{
    audio::{self, Audio},
    context::AppHandle,
    db::{self, Playlist, Track},
    lrc,
    lyrics::{self, LyricsQuery, LyricsResult},
    lyrics_ai::AiConfig,
    lyrics_jobs,
    player::{Phase, Player, Repeat},
    settings::{self, Palette, Settings},
    ytdlp::{self, OnlineTrack},
};
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, Stroke, Vec2};
use raw_window_handle::HasWindowHandle;
use std::{collections::HashMap, sync::mpsc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Home,
    Library,
    Liked,
    Online,
    Playlist(i64),
    Detail,
    Lyrics,
    Karaoke,
    Settings,
}
enum Message {
    Done(String),
    Saved(String, u64),
    Related(u64, Vec<Track>),
    Failure(String),
    Refresh,
    Online(u64, Vec<Track>),
    Home(Vec<Track>),
    Prepared(u64, Result<audio::Prepared, String>),
    Lyrics(String, Result<Option<LyricsResult>, String>),
    Preview(String, u64, LyricsResult),
    Settings(lyrics_jobs::SettingsView),
    Reviewed(String, u64, String),
    Aligned(String, u64, String),
    Media(souvlaki::MediaControlEvent),
}
#[derive(Clone)]
enum Confirmation {
    Track(i64),
    Playlist(i64),
    Lyrics(Track),
}
pub struct MusicApp {
    pub services: AppHandle,
    pub player: Player,
    pub audio: Audio,
    pub settings: Settings,
    pub page: Page,
    tracks: Vec<Track>,
    playlists: Vec<Playlist>,
    online: Vec<Track>,
    home: Vec<Track>,
    selected: Option<Track>,
    query: String,
    online_query: String,
    source: String,
    search_id: u64,
    lyric_cache: HashMap<String, Result<Option<LyricsResult>, String>>,
    lyric_pending: Vec<String>,
    editor: String,
    editor_track: String,
    editor_revision: u64,
    editor_dirty: bool,
    spelling: String,
    preview: Option<LyricsResult>,
    ai: AiConfig,
    api_key: String,
    settings_view: Option<lyrics_jobs::SettingsView>,
    palettes: Vec<Palette>,
    status: String,
    status_error: bool,
    busy: usize,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    media: Option<souvlaki::MediaControls>,
    recorded: bool,
    queue_open: bool,
    desk_open: bool,
    desk_tab: usize,
    controls: bool,
    karaoke_window: bool,
    karaoke_fullscreen: bool,
    mini: bool,
    island_expanded: bool,
    replace_source: bool,
    playlist_name: String,
    rename_name: String,
    confirm: Option<Confirmation>,
    last_position: f64,
    last_media: std::time::Instant,
    initialized: bool,
    last_draft_save: std::time::Instant,
}
impl MusicApp {
    pub fn new(cc: &eframe::CreationContext<'_>, services: AppHandle) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut fonts = egui::FontDefinitions::default();
        for path in [
            "C:/Windows/Fonts/segoeui.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ] {
            if let Ok(bytes) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("system".into(), egui::FontData::from_owned(bytes).into());
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "system".into());
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing = Vec2::new(10., 8.);
            s.spacing.button_padding = Vec2::new(12., 7.);
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(15.));
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(14.));
            s.text_styles
                .insert(egui::TextStyle::Heading, FontId::proportional(27.));
        });
        let mut app = Self::headless(services);
        let hwnd = cc.window_handle().ok().and_then(|h| match h.as_raw() {
            raw_window_handle::RawWindowHandle::Win32(w) => {
                Some(w.hwnd.get() as *mut std::ffi::c_void)
            }
            _ => None,
        });
        let config = souvlaki::PlatformConfig {
            dbus_name: "trantramy",
            display_name: "Trantramy",
            hwnd,
        };
        match souvlaki::MediaControls::new(config) {
            Ok(mut media) => {
                let tx = app.tx.clone();
                let ctx = cc.egui_ctx.clone();
                if let Err(e) = media.attach(move |event| {
                    let _ = tx.send(Message::Media(event));
                    ctx.request_repaint();
                }) {
                    app.fail(format!("Không đăng ký được phím media: {e:?}"));
                } else {
                    app.media = Some(media);
                }
            }
            Err(e) => app.fail(format!("Không khởi tạo được phím media: {e:?}")),
        }
        app
    }
    pub fn headless(services: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel();
        let settings_path = services.path().data.join("native-settings.json");
        let (settings, error) = match Settings::load(&settings_path) {
            Ok(s) => (s, String::new()),
            Err(e) => (Settings::default(), e),
        };
        let player = Player {
            shuffle: settings.shuffle,
            repeat: settings.repeat,
            ..Default::default()
        };
        let audio = Audio::default();
        *audio.eq.lock().unwrap() = settings.eq.clone();
        let mut app = Self {
            services,
            player,
            audio,
            settings,
            page: Page::Home,
            tracks: vec![],
            playlists: vec![],
            online: vec![],
            home: vec![],
            selected: None,
            query: String::new(),
            online_query: String::new(),
            source: "all".into(),
            search_id: 0,
            lyric_cache: HashMap::new(),
            lyric_pending: vec![],
            editor: String::new(),
            editor_track: String::new(),
            editor_revision: 0,
            editor_dirty: false,
            spelling: String::new(),
            preview: None,
            ai: AiConfig::default(),
            api_key: String::new(),
            settings_view: None,
            palettes: settings::palettes(),
            status: error.clone(),
            status_error: !error.is_empty(),
            busy: 0,
            tx,
            rx,
            media: None,
            recorded: false,
            queue_open: false,
            desk_open: false,
            desk_tab: 0,
            controls: true,
            karaoke_window: false,
            karaoke_fullscreen: false,
            mini: false,
            island_expanded: false,
            replace_source: false,
            playlist_name: String::new(),
            rename_name: String::new(),
            confirm: None,
            last_position: 0.,
            last_media: std::time::Instant::now(),
            initialized: false,
            last_draft_save: std::time::Instant::now(),
        };
        app.ai = app
            .services
            .state::<lyrics_jobs::LyricsService>()
            .config
            .lock()
            .map(|c| c.clone())
            .unwrap_or_default();
        app.refresh();
        app
    }
    fn fail(&mut self, error: impl Into<String>) {
        self.status = error.into();
        self.status_error = true;
    }
    fn notice(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.status_error = false;
    }
    fn task(
        &mut self,
        future: impl std::future::Future<Output = Result<Message, String>> + Send + 'static,
    ) {
        self.busy += 1;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let message = future.await.unwrap_or_else(Message::Failure);
            let _ = tx.send(message);
        });
    }
    fn refresh(&mut self) {
        let result = (|| {
            let c = self
                .services
                .state::<db::Db>()
                .inner()
                .0
                .lock()
                .map_err(|_| "Thư viện đang lỗi.".to_string())?;
            Ok::<_, String>((
                db::list_tracks(&c, "").map_err(|e| e.to_string())?,
                db::list_playlists(&c).map_err(|e| e.to_string())?,
            ))
        })();
        match result {
            Ok((tracks, playlists)) => {
                self.tracks = tracks;
                self.playlists = playlists;
            }
            Err(e) => self.fail(e),
        }
    }
    fn db_action(&mut self, f: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<()>) {
        let result = self
            .services
            .state::<db::Db>()
            .0
            .lock()
            .map_err(|_| "Thư viện đang lỗi.".to_string())
            .and_then(|c| f(&c).map_err(|e| e.to_string()));
        if let Err(e) = result {
            self.fail(e);
        } else {
            self.refresh();
        }
    }
    fn save_settings(&mut self) {
        self.settings.shuffle = self.player.shuffle;
        self.settings.repeat = self.player.repeat;
        self.settings.eq = self.audio.eq.lock().map(|e| e.clone()).unwrap_or_default();
        if let Err(e) = self
            .settings
            .save(&self.services.path().data.join("native-settings.json"))
        {
            self.fail(e);
        }
    }
    fn load_current(&mut self) {
        self.audio.stop();
        self.recorded = false;
        let Some(track) = self.player.current().cloned() else {
            return;
        };
        let token = self.player.generation;
        let app = self.services.clone();
        self.request_lyrics(&track);
        if let Some(media) = &mut self.media {
            let _ = media.set_metadata(souvlaki::MediaMetadata {
                title: Some(&track.title),
                artist: Some(&track.artist),
                album: Some(&track.album),
                duration: Some(Duration::from_secs_f64(track.duration.max(0.))),
                cover_url: track.cover_path.as_deref(),
            });
        }
        self.task(async move {
            let result = async {
                let mut path = track.path.clone();
                if path.starts_with("https://")
                    || path.starts_with("http://")
                    || track.source == "spotify"
                {
                    let mut url = track.url.clone().unwrap_or(path);
                    if track.source == "spotify" || url.contains("spotify.com") {
                        let results = ytdlp::search(
                            &app,
                            &format!("{} - {} official audio", track.artist, track.title),
                            "youtube",
                            1,
                        )
                        .await?;
                        url = results
                            .first()
                            .ok_or("Không tìm thấy bản âm thanh tương ứng.")?
                            .url
                            .clone();
                    }
                    path = ytdlp::resolve_stream(&app, &app.state(), &url).await?;
                }
                tokio::task::spawn_blocking(move || audio::prepare(&path))
                    .await
                    .map_err(|e| e.to_string())?
            }
            .await;
            Ok(Message::Prepared(token, result))
        });
    }
    fn play_list(&mut self, tracks: Vec<Track>, index: usize) {
        if self.player.start(tracks, index) {
            self.load_current();
        }
    }
    fn enqueue(&mut self, track: Track) {
        if self.player.enqueue(track) {
            self.load_current();
        } else {
            self.notice("Đã thêm vào hàng chờ.");
        }
    }
    fn toggle(&mut self) {
        if matches!(
            self.player.phase,
            Phase::Failed | Phase::Waiting | Phase::Idle
        ) && self.player.reload()
        {
            self.load_current();
            return;
        }
        if let Some(s) = &self.audio.sink {
            if s.is_paused() {
                s.play();
                self.player.phase = Phase::Playing;
            } else {
                s.pause();
                self.player.phase = Phase::Paused;
            }
        }
    }
    fn next(&mut self, natural: bool, failure: bool) {
        if self.player.next(natural, failure) {
            self.load_current();
        } else {
            self.audio.stop();
            if natural && !failure && !self.player.karaoke && self.settings.auto_related {
                let app = self.services.clone();
                let generation = self.player.generation;
                self.task(async move {
                    Ok(Message::Related(
                        generation,
                        crate::recommend::get_recommendations(&app, true)
                            .await?
                            .iter()
                            .map(online_track)
                            .collect(),
                    ))
                });
            }
        }
    }
    fn request_lyrics(&mut self, track: &Track) {
        if self.lyric_cache.contains_key(&track.path) || self.lyric_pending.contains(&track.path) {
            return;
        }
        self.lyric_pending.push(track.path.clone());
        let app = self.services.clone();
        let track = track.clone();
        self.task(async move {
            let key = track.path.clone();
            let result = lyrics_jobs::lyrics_read(app, track.into()).await;
            Ok(Message::Lyrics(key, result))
        });
    }
    fn draft_path(&self, key: &str) -> std::path::PathBuf {
        use sha2::{Digest, Sha256};
        self.services
            .path()
            .data
            .join("native-drafts")
            .join(format!("{:x}.lrc", Sha256::digest(key.as_bytes())))
    }
    fn persist_editor(&mut self) -> bool {
        if !self.editor_dirty || self.editor_track.is_empty() {
            return true;
        }
        let path = self.draft_path(&self.editor_track);
        let result = std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| e.to_string())
            .and_then(|_| atomic_text(&path, &self.editor));
        match result {
            Ok(()) => true,
            Err(e) => {
                self.fail(format!("Không lưu được bản nháp: {e}"));
                false
            }
        }
    }
    fn switch_editor(&mut self, track: &Track) -> bool {
        if self.editor_track == track.path {
            return true;
        }
        if !self.persist_editor() {
            return false;
        }
        let draft = self.draft_path(&track.path);
        let content = if draft.exists() {
            match lyrics::read_limited(&draft, lyrics::MAX_LYRICS)
                .and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
            {
                Ok(s) => Some(s),
                Err(e) => {
                    self.fail(format!("Không đọc được bản nháp: {e}"));
                    return false;
                }
            }
        } else {
            None
        };
        self.editor_dirty = content.is_some();
        self.editor = content.unwrap_or_default();
        self.spelling.clear();
        self.preview = None;
        self.editor_track = track.path.clone();
        self.editor_revision += 1;
        true
    }
    fn open_track(&mut self, track: Track, page: Page) {
        if !self.switch_editor(&track) {
            return;
        }
        self.request_lyrics(&track);
        self.selected = Some(track);
        self.page = page;
        self.sync_editor();
    }
    fn sync_editor(&mut self) {
        if !self.editor_dirty
            && self.editor.is_empty()
            && let Some(Ok(Some(result))) = self.lyric_cache.get(&self.editor_track)
        {
            self.editor = result.content.clone();
            self.spelling = lrc::parse(&self.editor)
                .lines
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    fn search(&mut self, replacement: bool) {
        self.search_id += 1;
        let id = self.search_id;
        let app = self.services.clone();
        let query = self.online_query.clone();
        let source = self.source.clone();
        self.replace_source = replacement;
        self.task(async move {
            if crate::spotify::parse_spotify_input(&query).is_some() {
                let collection = tokio::task::spawn_blocking(move || {
                    crate::spotify::fetch_spotify(&query, None, None)
                })
                .await
                .map_err(|e| e.to_string())??;
                let tracks = collection
                    .tracks
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| Track {
                        id: -(i as i64) - 1,
                        path: t.url.clone(),
                        url: Some(t.url),
                        title: t.title,
                        artist: t.artist,
                        album: t.album.unwrap_or_default(),
                        duration: t.duration as f64,
                        cover_path: t.thumbnail,
                        source: "spotify".into(),
                        liked: false,
                    })
                    .collect();
                return Ok(Message::Online(id, tracks));
            }
            let tracks = ytdlp::search(&app, &query, &source, 30).await?;
            Ok(Message::Online(
                id,
                tracks.iter().map(online_track).collect(),
            ))
        });
    }
    fn scan(&mut self, paths: Vec<std::path::PathBuf>) {
        let app = self.services.clone();
        self.task(async move {
            tokio::task::spawn_blocking(move || {
                let files: Vec<_> = paths
                    .iter()
                    .flat_map(|p| {
                        if p.is_dir() {
                            crate::scanner::collect_files(p)
                        } else if crate::scanner::is_audio(p) {
                            vec![p.clone()]
                        } else {
                            vec![]
                        }
                    })
                    .collect();
                let total = files.len();
                let covers = app.path().cache.join("covers");
                let mut failures = Vec::new();
                for (i, file) in files.iter().enumerate() {
                    let c = app
                        .state::<db::Db>()
                        .inner()
                        .0
                        .lock()
                        .map_err(|_| "Thư viện đang lỗi.")?;
                    if let Err(e) = crate::scanner::index_file(&c, file, &covers) {
                        failures.push(format!("{}: {e}", file.display()));
                    }
                    let _ = app.emit(
                        "scan-progress",
                        serde_json::json!({"done":i+1,"total":total}),
                    );
                }
                if !failures.is_empty() {
                    return Err(format!(
                        "Đã quét {total} tệp, lỗi {} tệp. {}",
                        failures.len(),
                        failures.first().unwrap()
                    ));
                }
                Ok(Message::Refresh)
            })
            .await
            .map_err(|e| e.to_string())?
        });
    }
    fn add_online(&mut self, track: Track, download: bool) {
        let app = self.services.clone();
        self.task(async move {
            let t = track.clone();
            let a = app.clone();
            let id = tokio::task::spawn_blocking(move || {
                let cover = t
                    .cover_path
                    .as_ref()
                    .and_then(|u| ytdlp::cache_thumbnail(&a.path().cache.join("covers"), u));
                let c = a
                    .state::<db::Db>()
                    .inner()
                    .0
                    .lock()
                    .map_err(|_| "Thư viện đang lỗi.")?;
                db::add_online_track(
                    &c,
                    t.url.as_deref().unwrap_or(&t.path),
                    (&t.title, &t.artist, &t.album),
                    t.duration,
                    cover.as_deref().or(t.cover_path.as_deref()),
                    &t.source,
                )
                .map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())??;
            if download {
                let mut url = track.url.clone().unwrap_or(track.path.clone());
                if track.source == "spotify" {
                    let results = ytdlp::search(
                        &app,
                        &format!("{} {} official audio", track.artist, track.title),
                        "youtube",
                        1,
                    )
                    .await?;
                    url = results
                        .first()
                        .ok_or("Không tìm thấy bản tương ứng.")?
                        .url
                        .clone();
                }
                let dir = app
                    .path()
                    .audio_dir()
                    .unwrap_or(app.path().data.clone())
                    .join("Trantramy");
                let path = ytdlp::download_audio(&app, &url, &dir, Some(id)).await?;
                db::set_track_file(
                    &*app
                        .state::<db::Db>()
                        .0
                        .lock()
                        .map_err(|_| "Thư viện đang lỗi.")?,
                    id,
                    &path.to_string_lossy(),
                    None,
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(Message::Refresh)
        });
    }
    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(message) = self.rx.try_recv() {
            if !matches!(message, Message::Media(_)) {
                self.busy = self.busy.saturating_sub(1);
            }
            match message {
                Message::Done(s) => self.notice(s),
                Message::Saved(key, revision) => {
                    if key == self.editor_track && revision == self.editor_revision {
                        self.editor_dirty = false;
                        let draft = self.draft_path(&key);
                        if draft.exists()
                            && let Err(e) = std::fs::remove_file(draft)
                        {
                            self.fail(format!("Đã lưu lời nhưng chưa dọn được bản nháp: {e}"));
                            continue;
                        }
                    }
                    self.notice("Đã lưu lời bài hát.");
                }
                Message::Related(generation, tracks) => {
                    if generation == self.player.generation
                        && self.player.phase == Phase::Idle
                        && !self.player.karaoke
                        && !tracks.is_empty()
                    {
                        self.play_list(tracks, 0);
                    }
                }
                Message::Failure(e) => {
                    self.refresh();
                    self.fail(e);
                }
                Message::Refresh => {
                    self.refresh();
                    self.notice("Đã cập nhật thư viện.");
                }
                Message::Online(id, tracks) => {
                    if id == self.search_id {
                        self.online = tracks;
                        self.notice(format!("{} kết quả", self.online.len()));
                    }
                }
                Message::Home(tracks) => {
                    self.home = tracks;
                }
                Message::Prepared(token, result) => {
                    if token != self.player.generation {
                        continue;
                    }
                    match result.and_then(|p| self.audio.load(p, self.settings.volume)) {
                        Ok(()) => {
                            self.player.accept(token);
                        }
                        Err(e) => {
                            self.fail(e.clone());
                            self.next(false, true);
                            self.player.error = e;
                        }
                    }
                }
                Message::Lyrics(key, result) => {
                    self.lyric_pending.retain(|p| p != &key);
                    self.lyric_cache.insert(key, result);
                    self.sync_editor();
                }
                Message::Preview(key, revision, result) => {
                    if key == self.editor_track && revision == self.editor_revision {
                        self.preview = Some(result);
                        self.notice("Đã tạo bản xem trước. Kiểm tra rồi lưu khi phù hợp.");
                    } else {
                        self.notice("Bản xem trước cũ đã bỏ qua vì bạn đổi bài hoặc sửa lời.");
                    }
                }
                Message::Settings(view) => {
                    self.ai = view.config.clone();
                    self.settings_view = Some(view);
                }
                Message::Reviewed(key, revision, content)
                | Message::Aligned(key, revision, content) => {
                    if key == self.editor_track && revision == self.editor_revision {
                        self.preview = Some(LyricsResult::new(content, "review", true));
                    } else {
                        self.notice("Kết quả cũ đã bỏ qua vì nội dung đang sửa thay đổi.");
                    }
                }
                Message::Media(event) => self.media_event(event, ctx),
            }
        }
        for (name, value) in self.services.drain_events() {
            if name == "lyrics-updated" {
                if let Some(key) = value.as_str() {
                    self.lyric_cache.remove(key);
                    if let Some(t) = self
                        .tracks
                        .iter()
                        .chain(self.player.queue.iter())
                        .find(|t| t.path == key)
                        .cloned()
                    {
                        self.request_lyrics(&t);
                    }
                }
            } else if name == "lyrics-progress" {
                self.notice(value["message"].as_str().unwrap_or("Đang xử lý lời…"));
            } else if name == "scan-progress" {
                self.notice(format!("Đang quét: {} / {}", value["done"], value["total"]));
            } else if name == "download-progress" {
                self.notice(format!(
                    "Tải nhạc: {:.0}% · {}",
                    value["percent"].as_f64().unwrap_or(0.),
                    value["status"].as_str().unwrap_or("")
                ));
            }
        }
        if self.player.phase == Phase::Playing {
            self.last_position = self.audio.position();
            if !self.recorded && self.last_position >= 5f64.max((self.duration() * 0.5).min(30.)) {
                self.record(false);
                self.recorded = true;
            }
            if self.audio.sink.as_ref().is_some_and(|s| s.empty()) {
                self.record(true);
                self.next(true, false);
            }
        }
        if self.editor_dirty && self.last_draft_save.elapsed() > Duration::from_secs(2) {
            self.last_draft_save = std::time::Instant::now();
            self.persist_editor();
        }
        if self.last_media.elapsed() > Duration::from_secs(1) {
            self.last_media = std::time::Instant::now();
            let phase = self.player.phase;
            let progress = Some(souvlaki::MediaPosition(Duration::from_secs_f64(
                self.audio.position().max(0.),
            )));
            if let Some(media) = &mut self.media {
                let _ = media.set_playback(match phase {
                    Phase::Playing => souvlaki::MediaPlayback::Playing { progress },
                    Phase::Paused => souvlaki::MediaPlayback::Paused { progress },
                    _ => souvlaki::MediaPlayback::Stopped,
                });
            }
        }
    }
    fn media_event(&mut self, event: souvlaki::MediaControlEvent, ctx: &egui::Context) {
        use souvlaki::MediaControlEvent as E;
        match event {
            E::Toggle => self.toggle(),
            E::Play => {
                if self.player.phase != Phase::Playing {
                    self.toggle();
                }
            }
            E::Pause => {
                if self.player.phase == Phase::Playing {
                    self.toggle();
                }
            }
            E::Next => self.next(false, false),
            E::Previous => {
                if self.player.previous() {
                    self.load_current();
                }
            }
            E::Stop => {
                self.audio.stop();
                self.player.phase = Phase::Idle;
            }
            E::SetPosition(p) => {
                if let Err(e) = self.audio.seek(p.0.as_secs_f64()) {
                    self.fail(e);
                }
            }
            E::SeekBy(direction, d) => {
                let sign = if direction == souvlaki::SeekDirection::Forward {
                    1.
                } else {
                    -1.
                };
                if let Err(e) = self
                    .audio
                    .seek((self.audio.position() + sign * d.as_secs_f64()).max(0.))
                {
                    self.fail(e);
                }
            }
            E::SetVolume(v) => {
                self.settings.volume = (v as f32).clamp(0., 1.);
                if let Some(s) = &self.audio.sink {
                    s.set_volume(self.settings.volume);
                }
                self.save_settings();
            }
            E::SetShuffle(s) => {
                self.player.shuffle = s;
                self.player.reorder();
                self.save_settings();
            }
            E::OpenUri(uri) => {
                self.online_query = uri;
                self.page = Page::Online;
                self.search(false);
            }
            E::Raise => ctx.send_viewport_cmd(egui::ViewportCommand::Focus),
            E::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }
    }
    fn record(&mut self, completed: bool) {
        if let Some(t) = self.player.current() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let result = self
                .services
                .state::<db::Db>()
                .0
                .lock()
                .map_err(|_| "Thư viện đang lỗi.".to_owned())
                .and_then(|c| {
                    db::record_play(
                        &c,
                        &t.title,
                        &t.artist,
                        &t.source,
                        t.url.as_deref(),
                        now,
                        completed,
                    )
                    .map_err(|e| e.to_string())
                });
            if let Err(e) = result {
                self.fail(e);
            }
        }
    }
    fn duration(&self) -> f64 {
        if self.audio.duration > 0. {
            self.audio.duration
        } else {
            self.player.current().map_or(0., |t| t.duration)
        }
    }
    fn accent(&self, ctx: &egui::Context) -> Color32 {
        settings::readable(
            ctx.style().visuals.selection.bg_fill,
            ctx.style().visuals.panel_fill,
        )
    }
    fn open_karaoke(&mut self) {
        self.page = Page::Karaoke;
        self.player.karaoke = true;
        self.player.reorder();
    }
    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.);
        ui.label(RichText::new("Trantramy").size(24.).strong());
        ui.label("Âm nhạc của bạn");
        ui.add_space(24.);
        for (page, label) in [
            (Page::Home, "Trang chủ"),
            (Page::Library, "Thư viện"),
            (Page::Liked, "Yêu thích"),
            (Page::Online, "Trực tuyến"),
            (Page::Karaoke, "Studio Karaoke"),
            (Page::Settings, "Cài đặt"),
        ] {
            if ui.selectable_label(self.page == page, label).clicked() {
                if page == Page::Karaoke {
                    self.open_karaoke();
                } else {
                    self.page = page;
                }
            }
        }
        ui.add_space(20.);
        ui.separator();
        ui.label(RichText::new("Danh sách phát").strong());
        for playlist in self.playlists.clone() {
            if ui
                .selectable_label(
                    self.page == Page::Playlist(playlist.id),
                    format!("{} · {}", playlist.name, playlist.count),
                )
                .clicked()
            {
                self.page = Page::Playlist(playlist.id);
                self.rename_name = playlist.name;
            }
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.playlist_name)
                    .hint_text("Danh sách mới")
                    .desired_width(110.),
            );
            if ui.button("Tạo").clicked() {
                let name = self.playlist_name.trim().to_owned();
                if !name.is_empty() {
                    self.db_action(|c| db::create_playlist(c, &name).map(|_| ()));
                    self.playlist_name.clear();
                }
            }
        });
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.label(RichText::new("2.0.0 · Rust native").small());
            if ui.button("Thêm thư mục nhạc").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                self.scan(vec![path]);
            }
        });
    }
    fn header(&mut self, ui: &mut egui::Ui) {
        let compact = ui.available_width() < 700.;
        ui.horizontal_wrapped(|ui| {
            if ui.available_width() < 900. {
                egui::ComboBox::from_id_salt("navigation")
                    .selected_text(page_name(self.page))
                    .show_ui(ui, |ui| {
                        for (page, label) in [
                            (Page::Home, "Trang chủ"),
                            (Page::Library, "Thư viện"),
                            (Page::Liked, "Yêu thích"),
                            (Page::Online, "Trực tuyến"),
                            (Page::Karaoke, "Studio Karaoke"),
                            (Page::Settings, "Cài đặt"),
                        ] {
                            if ui.selectable_value(&mut self.page, page, label).clicked()
                                && page == Page::Karaoke
                            {
                                self.open_karaoke();
                            }
                        }
                        for p in self.playlists.clone() {
                            ui.selectable_value(&mut self.page, Page::Playlist(p.id), p.name);
                        }
                    });
            }
            if !compact {
                ui.heading(page_name(self.page));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Hàng chờ").clicked() {
                    self.queue_open = !self.queue_open;
                }
                if ui.button("Studio").clicked() {
                    self.open_karaoke();
                }
                if self.busy > 0 {
                    ui.spinner();
                    ui.label(format!("{} tác vụ", self.busy));
                }
            });
        });
        ui.add_space(if compact { 2. } else { 12. });
        ui.separator();
        ui.add_space(if compact { 2. } else { 12. });
    }
    fn transport_options(&mut self, ui: &mut egui::Ui, track: Option<Track>) {
        if ui
            .selectable_label(self.player.shuffle, "Ngẫu nhiên")
            .clicked()
        {
            self.player.shuffle = !self.player.shuffle;
            self.player.reorder();
            self.save_settings();
        }
        if ui
            .button(match self.player.repeat {
                Repeat::Off => "Không lặp",
                Repeat::All => "Lặp tất cả",
                Repeat::One => "Lặp một bài",
            })
            .clicked()
        {
            self.player.repeat = match self.player.repeat {
                Repeat::Off => Repeat::All,
                Repeat::All => Repeat::One,
                Repeat::One => Repeat::Off,
            };
            self.save_settings();
        }
        if ui
            .add_enabled(track.is_some(), egui::Button::new("Lời bài hát"))
            .clicked()
            && let Some(t) = track.clone()
        {
            self.open_track(t, Page::Lyrics);
        }
        ui.label("Âm lượng");
        if ui
            .add(egui::Slider::new(&mut self.settings.volume, 0.0..=1.0).show_value(false))
            .changed()
        {
            if let Some(s) = &self.audio.sink {
                s.set_volume(self.settings.volume);
            }
            self.save_settings();
        }
    }
    fn transport(&mut self, ui: &mut egui::Ui) {
        let duration = self.duration();
        let mut position = self.audio.position();
        let track = self.player.current().cloned();
        ui.horizontal_wrapped(|ui| {
            if let Some(t) = &track {
                let title = ui.add(
                    egui::Label::new(RichText::new(&t.title).strong())
                        .truncate()
                        .sense(Sense::click()),
                );
                if title.clicked() {
                    self.open_track(t.clone(), Page::Detail);
                }
                ui.label(RichText::new(&t.artist).small());
            } else {
                ui.label("Chọn một bài hát để bắt đầu");
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(track.is_some(), egui::Button::new("Bài trước"))
                .clicked()
            {
                if self.audio.position() > 3. {
                    if let Err(e) = self.audio.seek(0.) {
                        self.fail(e);
                    }
                } else if self.player.previous() {
                    self.load_current();
                }
            }
            let playing = self.player.phase == Phase::Playing;
            if ui
                .add_enabled(
                    track.is_some() && self.player.phase != Phase::Loading,
                    egui::Button::new(if playing { "Tạm dừng" } else { "Phát" }),
                )
                .clicked()
            {
                self.toggle();
            }
            if ui
                .add_enabled(track.is_some(), egui::Button::new("Bài tiếp"))
                .clicked()
            {
                self.next(false, false);
            }
            if ui.available_width() < 370. {
                ui.menu_button("Tùy chọn", |ui| {
                    self.transport_options(ui, track.clone())
                });
            } else {
                self.transport_options(ui, track.clone());
            }
        });
        ui.horizontal(|ui| {
            ui.monospace(time(position));
            let width = (ui.available_width() - 60.).max(20.);
            let response = ui.add_sized(
                [width, 20.],
                egui::Slider::new(&mut position, 0.0..=duration.max(1.)).show_value(false),
            );
            if response.changed()
                && let Err(e) = self.audio.seek(position)
            {
                self.fail(e);
            }
            ui.monospace(time(duration));
        });
        if self.player.phase == Phase::Loading {
            ui.label("Đang mở nguồn âm thanh…");
        }
        if !self.player.error.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &self.player.error);
        }
    }
    fn filtered_tracks(&mut self) -> Vec<Track> {
        let query = self.query.to_lowercase();
        let tracks = if let Page::Playlist(id) = self.page {
            self.services
                .state::<db::Db>()
                .0
                .lock()
                .ok()
                .and_then(|c| db::get_playlist(&c, id).ok())
                .unwrap_or_default()
        } else {
            self.tracks.clone()
        };
        tracks
            .into_iter()
            .filter(|t| {
                (self.page != Page::Liked || t.liked)
                    && (query.is_empty()
                        || format!("{} {} {}", t.title, t.artist, t.album)
                            .to_lowercase()
                            .contains(&query))
            })
            .collect()
    }
    fn track_list(&mut self, ui: &mut egui::Ui, tracks: &[Track], online: bool) {
        if tracks.is_empty() {
            ui.add_space(24.);
            ui.heading(if online {
                "Chưa có kết quả"
            } else {
                "Chưa có bài hát"
            });
            ui.label(if online {
                "Nhập tên bài hoặc dán liên kết YouTube, SoundCloud, Spotify rồi tìm."
            } else {
                "Thêm thư mục nhạc hoặc tìm nhạc trực tuyến để tạo thư viện."
            });
            return;
        }
        let narrow = ui.available_width() < 700.;
        egui::ScrollArea::vertical()
            .id_salt(("tracks", format!("{:?}", self.page)))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, track) in tracks.iter().enumerate() {
                    ui.push_id((&track.path, i), |ui| {
                        ui.horizontal(|ui| {
                            let current =
                                self.player.current().is_some_and(|t| t.path == track.path);
                            if ui
                                .button(if current && self.player.phase == Phase::Playing {
                                    "Dừng"
                                } else {
                                    "Phát"
                                })
                                .clicked()
                            {
                                if current {
                                    self.toggle();
                                } else {
                                    self.play_list(tracks.to_vec(), i);
                                }
                            }
                            if !narrow && let Some(cover) = &track.cover_path {
                                ui.add(
                                    egui::Image::new(uri(cover))
                                        .fit_to_exact_size(Vec2::splat(38.)),
                                );
                            }
                            let meta_width =
                                (ui.available_width() - if narrow { 150. } else { 240. }).max(70.);
                            ui.allocate_ui_with_layout(
                                Vec2::new(meta_width, 44.),
                                Layout::top_down(Align::LEFT),
                                |ui| {
                                    let response = ui.add(
                                        egui::Label::new(
                                            RichText::new(&track.title).strong().color(
                                                if current {
                                                    self.accent(ui.ctx())
                                                } else {
                                                    ui.visuals().text_color()
                                                },
                                            ),
                                        )
                                        .truncate()
                                        .sense(Sense::click()),
                                    );
                                    if response.double_clicked() {
                                        self.play_list(tracks.to_vec(), i);
                                    } else if response.clicked() {
                                        self.open_track(track.clone(), Page::Detail);
                                    }
                                    ui.add(
                                        egui::Label::new(RichText::new(&track.artist).small())
                                            .truncate(),
                                    );
                                },
                            );
                            ui.monospace(time(track.duration));
                            if ui
                                .button("Gọi bài")
                                .on_hover_text("Thêm bài này vào cuối hàng chờ")
                                .clicked()
                            {
                                self.enqueue(track.clone());
                            }
                            ui.menu_button("Thêm", |ui| {
                                if ui.button("Chi tiết / lời bài hát").clicked() {
                                    self.open_track(track.clone(), Page::Detail);
                                    ui.close_menu();
                                }
                                if ui
                                    .button(if track.liked {
                                        "Bỏ yêu thích"
                                    } else {
                                        "Yêu thích"
                                    })
                                    .clicked()
                                {
                                    if track.id > 0 {
                                        self.db_action(|c| {
                                            db::toggle_like_track(c, track.id).map(|_| ())
                                        });
                                    } else {
                                        self.add_online(track.clone(), false);
                                    }
                                    ui.close_menu();
                                }
                                if online && ui.button("Lưu vào thư viện").clicked() {
                                    self.add_online(track.clone(), false);
                                    ui.close_menu();
                                }
                                if track.path.starts_with("http")
                                    && ui.button("Tải nhạc về máy").clicked()
                                {
                                    self.add_online(track.clone(), true);
                                    ui.close_menu();
                                }
                                if track.id > 0 {
                                    ui.menu_button("Thêm vào danh sách", |ui| {
                                        for p in self.playlists.clone() {
                                            if ui.button(p.name).clicked() {
                                                self.db_action(|c| {
                                                    db::add_to_playlist(c, p.id, track.id)
                                                });
                                                ui.close_menu();
                                            }
                                        }
                                    });
                                    if let Page::Playlist(id) = self.page
                                        && ui.button("Bỏ khỏi danh sách này").clicked()
                                    {
                                        self.db_action(|c| {
                                            db::remove_from_playlist(c, id, track.id)
                                        });
                                        ui.close_menu();
                                    }
                                    if ui.button("Xóa khỏi thư viện").clicked() {
                                        self.confirm = Some(Confirmation::Track(track.id));
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                        ui.separator();
                    });
                }
            });
    }
    fn library_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Tìm bài, nghệ sĩ, album")
                    .desired_width(250.),
            );
            if ui.button("Thêm nhạc").clicked()
                && let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Âm thanh", crate::scanner::EXTS)
                    .pick_files()
            {
                self.scan(paths);
            }
            if ui.button("Thêm thư mục").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                self.scan(vec![path]);
            }
        });
        if let Page::Playlist(id) = self.page {
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.rename_name).desired_width(200.));
                if ui.button("Đổi tên").clicked() {
                    let name = self.rename_name.trim().to_owned();
                    if !name.is_empty() {
                        self.db_action(|c| db::rename_playlist(c, id, &name));
                    }
                }
                if ui.button("Ảnh danh sách").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Ảnh", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                {
                    let dest = self.services.path().data.join("playlist_covers");
                    match copy_image(&path, &dest) {
                        Ok(p) => self.db_action(|c| db::set_playlist_cover(c, id, Some(&p))),
                        Err(e) => self.fail(e),
                    }
                }
                if ui.button("Xóa danh sách").clicked() {
                    self.confirm = Some(Confirmation::Playlist(id));
                }
            });
        }
        let tracks = self.filtered_tracks();
        ui.horizontal(|ui| {
            ui.label(format!("{} bài hát", tracks.len()));
            if ui
                .add_enabled(!tracks.is_empty(), egui::Button::new("Phát tất cả"))
                .clicked()
            {
                self.play_list(tracks.clone(), 0);
            }
            if ui
                .add_enabled(!tracks.is_empty(), egui::Button::new("Phát ngẫu nhiên"))
                .clicked()
            {
                self.player.shuffle = true;
                self.play_list(tracks.clone(), 0);
            }
        });
        ui.add_space(8.);
        self.track_list(ui, &tracks, false);
    }
    fn home_ui(&mut self, ui: &mut egui::Ui) {
        let title = self
            .player
            .current()
            .map_or("Bản nhạc tiếp theo, bạn chọn nhé.", |t| {
                t.title.as_str()
            })
            .to_owned();
        ui.horizontal(|ui| {
            if ui.available_width() > 650. {
                self.vinyl(ui, 200.);
            }
            ui.vertical(|ui| {
                ui.add_space(10.);
                ui.label(RichText::new(title).size(28.).strong());
                ui.label("Thư viện riêng · nhạc trực tuyến · Studio lời bài hát");
                ui.add_space(16.);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Mở thư viện").clicked() {
                        self.page = Page::Library;
                    }
                    if ui.button("Tìm nhạc trực tuyến").clicked() {
                        self.page = Page::Online;
                    }
                    if ui.button("Vào Studio").clicked() {
                        self.open_karaoke();
                    }
                });
            });
        });
        ui.add_space(20.);
        ui.horizontal(|ui| {
            ui.heading("Dành cho bạn");
            if ui.button("Làm mới").clicked() {
                let app = self.services.clone();
                self.task(async move {
                    Ok(Message::Home(
                        crate::recommend::get_recommendations(&app, true)
                            .await?
                            .iter()
                            .map(online_track)
                            .collect(),
                    ))
                });
            }
        });
        let tracks = if self.home.is_empty() {
            self.tracks.iter().rev().take(16).cloned().collect()
        } else {
            self.home.clone()
        };
        self.track_list(ui, &tracks, !self.home.is_empty());
    }
    fn online_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(
            "Tìm nhạc trên YouTube / SoundCloud hoặc dán liên kết bài, playlist, album Spotify.",
        );
        ui.horizontal_wrapped(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.online_query)
                    .hint_text("Tên bài, nghệ sĩ hoặc liên kết")
                    .desired_width((ui.available_width() - 220.).max(180.)),
            );
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            egui::ComboBox::from_id_salt("online-source")
                .selected_text(match self.source.as_str() {
                    "youtube" => "YouTube",
                    "soundcloud" => "SoundCloud",
                    _ => "Tất cả",
                })
                .show_ui(ui, |ui| {
                    for (id, name) in [
                        ("all", "Tất cả"),
                        ("youtube", "YouTube"),
                        ("soundcloud", "SoundCloud"),
                    ] {
                        ui.selectable_value(&mut self.source, id.into(), name);
                    }
                });
            if ui.button("Tìm").clicked() || enter {
                self.search(false);
            }
        });
        let tracks = self.online.clone();
        self.track_list(ui, &tracks, true);
    }
    fn queue_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Hàng chờ");
            if ui.button("Xóa hàng chờ").clicked() {
                self.player.clear();
                self.audio.stop();
            }
        });
        if self.player.phase == Phase::Waiting {
            ui.label("Đã hát hết lượt. Gọi thêm bài để tiếp tục.");
        }
        let current = self.player.index;
        egui::ScrollArea::vertical()
            .id_salt("queue")
            .show(ui, |ui| {
                for (i, t) in self.player.queue.clone().iter().enumerate() {
                    ui.push_id(i, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(if current == Some(i) {
                                "Đang phát"
                            } else if current.is_some_and(|n| i < n) {
                                "Đã phát"
                            } else {
                                "Chờ hát"
                            });
                            if ui
                                .add(egui::Label::new(&t.title).truncate().sense(Sense::click()))
                                .double_clicked()
                            {
                                self.play_list(self.player.queue.clone(), i);
                            }
                            if current.is_some_and(|n| i > n) && ui.button("Ưu tiên").clicked() {
                                self.player.prioritize(i);
                            }
                            if ui.button("Bỏ bài").clicked() && self.player.remove(i) {
                                if self.player.current().is_some() {
                                    self.load_current();
                                } else {
                                    self.audio.stop();
                                }
                            }
                        });
                        ui.separator();
                    });
                }
            });
    }
    fn detail_ui(&mut self, ui: &mut egui::Ui) {
        let Some(track) = self.selected.clone() else {
            ui.label("Chọn bài hát trong thư viện để xem chi tiết.");
            return;
        };
        ui.horizontal(|ui| {
            if let Some(cover) = &track.cover_path {
                ui.add(egui::Image::new(uri(cover)).fit_to_exact_size(Vec2::splat(120.)));
            }
            ui.vertical(|ui| {
                ui.heading(&track.title);
                ui.label(&track.artist);
                ui.label(&track.album);
                ui.label(format!("{} · {}", track.source, time(track.duration)));
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Phát").clicked() {
                        self.play_list(vec![track.clone()], 0);
                    }
                    if ui.button("Gọi bài").clicked() {
                        self.enqueue(track.clone());
                    }
                    if ui.button("Sửa lời").clicked() {
                        self.page = Page::Lyrics;
                    }
                    if track.id > 0
                        && ui
                            .button(if track.liked {
                                "Bỏ yêu thích"
                            } else {
                                "Yêu thích"
                            })
                            .clicked()
                    {
                        self.db_action(|c| db::toggle_like_track(c, track.id).map(|_| ()));
                        self.selected = self.tracks.iter().find(|t| t.id == track.id).cloned();
                    }
                });
            });
        });
        ui.add_space(20.);
        ui.label(RichText::new("Lời bài hát").size(21.).strong());
        if let Some(result) = self.lyric_cache.get(&track.path).cloned() {
            match result {
                Ok(Some(result)) => {
                    if result.draft_only {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "Bản nháp chưa xác nhận. Kiểm tra trước khi lưu.",
                        );
                    }
                    let parsed = lrc::parse(&result.content);
                    let position = if self.player.current().is_some_and(|t| t.path == track.path) {
                        self.audio.position()
                    } else {
                        -1.
                    };
                    egui::ScrollArea::vertical()
                        .id_salt("detail-lyrics")
                        .show(ui, |ui| {
                            for line in parsed.lines {
                                let response = ui.add(
                                    egui::Label::new(RichText::new(&line.text).size(19.))
                                        .sense(Sense::click()),
                                );
                                if response.clicked()
                                    && position >= 0.
                                    && line.time >= 0.
                                    && !result.draft_only
                                    && let Err(e) = self.audio.seek(line.time)
                                {
                                    self.fail(e);
                                }
                            }
                        });
                }
                Ok(None) => {
                    ui.label("Bản nhạc chưa có lời. Mở Sửa lời để thêm hoặc tra cứu.");
                }
                Err(e) => {
                    ui.colored_label(ui.visuals().error_fg_color, e);
                    if ui.button("Thử lại").clicked() {
                        self.lyric_cache.remove(&track.path);
                        self.request_lyrics(&track);
                    }
                }
            }
        } else {
            ui.spinner();
            ui.label("Đang tải lời…");
        }
    }
}
include!("app_studio.rs");
include!("app_shell.rs");
