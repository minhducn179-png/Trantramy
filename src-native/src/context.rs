use crate::{
    db::Db,
    lyrics_jobs::LyricsService,
    recommend::RecCache,
    ytdlp::{SearchCache, StreamCache},
};
use serde::Serialize;
use std::{
    collections::VecDeque,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct Services {
    pub db: Db,
    pub lyrics: LyricsService,
    pub streams: StreamCache,
    pub search: SearchCache,
    pub recommendations: RecCache,
    pub paths: Paths,
    events: Mutex<VecDeque<(String, serde_json::Value)>>,
}
#[derive(Clone)]
pub struct AppHandle(pub Arc<Services>);
#[derive(Clone)]
pub struct Paths {
    pub data: PathBuf,
    pub cache: PathBuf,
}
impl Paths {
    pub fn app_data_dir(&self) -> Result<PathBuf, String> {
        Ok(self.data.clone())
    }
    pub fn app_cache_dir(&self) -> Result<PathBuf, String> {
        Ok(self.cache.clone())
    }
    pub fn audio_dir(&self) -> Result<PathBuf, String> {
        dirs::audio_dir().ok_or_else(|| "Không tìm thấy thư mục nhạc.".into())
    }
}
pub trait Service {
    fn get(services: &Services) -> &Self;
}
pub struct State<'a, T>(&'a T);
impl<'a, T> State<'a, T> {
    pub fn inner(&self) -> &'a T {
        self.0
    }
}
impl<T> Deref for State<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
macro_rules! service {
    ($ty:ty, $field:ident) => {
        impl Service for $ty {
            fn get(s: &Services) -> &Self {
                &s.$field
            }
        }
    };
}
service!(Db, db);
service!(LyricsService, lyrics);
service!(StreamCache, streams);
service!(SearchCache, search);
service!(RecCache, recommendations);
impl AppHandle {
    pub fn open(paths: Paths) -> Result<Self, String> {
        std::fs::create_dir_all(&paths.data).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&paths.cache).map_err(|e| e.to_string())?;
        let database = paths.data.join("dlc.db");
        if !database.exists()
            && let Some(parent) = paths.data.parent()
        {
            let legacy = parent.join("com.miu.player/library.db");
            if legacy.is_file() {
                std::fs::copy(legacy, &database).map_err(|e| e.to_string())?;
            }
        }
        Ok(Self(Arc::new(Services {
            db: Db(Mutex::new(
                crate::db::open(&database).map_err(|e| e.to_string())?,
            )),
            lyrics: LyricsService::new(paths.data.join("lyrics"))?,
            streams: StreamCache::new(),
            search: SearchCache::new(),
            recommendations: RecCache::new(),
            paths,
            events: Mutex::new(VecDeque::new()),
        })))
    }
    pub fn system() -> Result<Self, String> {
        let data = if let Some(p) = std::env::var_os("TRANTRAMY_DATA_DIR") {
            PathBuf::from(p)
        } else {
            dirs::data_dir()
                .ok_or("Không tìm thấy thư mục dữ liệu.")?
                .join("com.trantramy.player")
        };
        let cache = dirs::cache_dir()
            .ok_or("Không tìm thấy thư mục cache.")?
            .join("com.trantramy.player");
        Self::open(Paths { data, cache })
    }
    pub fn path(&self) -> &Paths {
        &self.0.paths
    }
    pub fn state<T: Service>(&self) -> State<'_, T> {
        State(T::get(&self.0))
    }
    pub fn try_state<T: Service>(&self) -> Option<State<'_, T>> {
        Some(self.state())
    }
    pub fn emit<T: Serialize>(&self, name: &str, payload: T) -> Result<(), String> {
        let payload = serde_json::to_value(payload).map_err(|e| e.to_string())?;
        let mut queue = self.0.events.lock().map_err(|_| "Hàng sự kiện đang lỗi.")?;
        // ponytail: bounded event buffer, use per-consumer channels if more windows own state.
        if queue.len() == 512 {
            queue.pop_front();
        }
        queue.push_back((name.to_owned(), payload));
        Ok(())
    }
    pub fn drain_events(&self) -> Vec<(String, serde_json::Value)> {
        self.0
            .events
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default()
    }
}
