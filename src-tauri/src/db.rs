use rusqlite::{Connection, OptionalExtension, Result, params};
use serde::Serialize;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

#[derive(Serialize, Clone, Debug)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub cover_path: Option<String>,
    pub source: String,
    pub url: Option<String>,
    pub liked: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub count: i64,
    pub cover_path: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct HistoryRow {
    pub title: String,
    pub artist: String,
    pub source: String,
    pub url: Option<String>,
    pub played_at: i64,
    pub completed: bool,
}

pub fn open(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    init(&conn)?;
    Ok(conn)
}

pub fn init(conn: &Connection) -> Result<()> {
    init_schema(conn)?;
    // Safe migrations for existing databases.
    let _ = conn.execute("ALTER TABLE playlists ADD COLUMN cover_path TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE tracks ADD COLUMN source TEXT NOT NULL DEFAULT 'local'",
        [],
    );
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN url TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE tracks ADD COLUMN liked INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE history ADD COLUMN completed INTEGER NOT NULL DEFAULT 0",
        [],
    );
    Ok(())
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS tracks(
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT NOT NULL,
            duration REAL NOT NULL,
            cover_path TEXT,
            source TEXT NOT NULL DEFAULT 'local',
            url TEXT,
            liked INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS playlists(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS playlist_tracks(
            playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            PRIMARY KEY(playlist_id, track_id)
        );
        CREATE TABLE IF NOT EXISTS history(
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            source TEXT NOT NULL,
            url TEXT,
            played_at INTEGER NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0
        );",
    )
}

pub fn upsert_track(
    conn: &Connection,
    path: &str,
    title: &str,
    artist: &str,
    album: &str,
    duration: f64,
    cover: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO tracks(path,title,artist,album,duration,cover_path,source) VALUES(?1,?2,?3,?4,?5,?6,'local')
         ON CONFLICT(path) DO UPDATE SET title=?2,artist=?3,album=?4,duration=?5,cover_path=COALESCE(?6,tracks.cover_path)",
        params![path, title, artist, album, duration, cover],
    )?;
    Ok(())
}

/// Inserts or updates an online track (streamable or downloaded) and returns its track id.
pub fn add_online_track(
    conn: &Connection,
    url: &str,
    metadata: (&str, &str, &str),
    duration: f64,
    cover: Option<&str>,
    source: &str,
) -> Result<i64> {
    let (title, artist, album) = metadata;
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM tracks WHERE url=?1 OR path=?1 LIMIT 1",
            params![url],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
    {
        conn.execute(
            "UPDATE tracks SET title=?2, artist=?3, album=?4, duration=?5,
             cover_path=COALESCE(?6, cover_path), source=?7, url=?1 WHERE id=?8",
            params![url, title, artist, album, duration, cover, source, id],
        )?;
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO tracks(path,title,artist,album,duration,cover_path,source,url)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?1)",
        params![url, title, artist, album, duration, cover, source],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Marks an online track as downloaded by pointing `path` to the local file while preserving `url`.
pub fn set_track_file(
    conn: &Connection,
    id: i64,
    local_path: &str,
    cover: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE tracks SET path=?2, cover_path=COALESCE(?3, cover_path) WHERE id=?1",
        params![id, local_path, cover],
    )?;
    Ok(())
}

fn row_to_track(r: &rusqlite::Row) -> Result<Track> {
    Ok(Track {
        id: r.get(0)?,
        path: r.get(1)?,
        title: r.get(2)?,
        artist: r.get(3)?,
        album: r.get(4)?,
        duration: r.get(5)?,
        cover_path: r.get(6)?,
        source: r
            .get::<_, Option<String>>(7)?
            .unwrap_or_else(|| "local".into()),
        url: r.get(8)?,
        liked: r.get::<_, Option<i64>>(9)?.unwrap_or(0) != 0,
    })
}

const COLS: &str = "id,path,title,artist,album,duration,cover_path,source,url,liked";

pub fn get_track(conn: &Connection, id: i64) -> Result<Option<Track>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM tracks WHERE id=?1"),
        params![id],
        row_to_track,
    )
    .optional()
}

pub fn list_tracks(conn: &Connection, query: &str) -> Result<Vec<Track>> {
    let like = format!("%{}%", query);
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM tracks WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
         ORDER BY artist, album, title"
    ))?;
    let rows = stmt.query_map(params![like], row_to_track)?;
    rows.collect()
}

pub fn delete_track(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tracks WHERE id=?1", params![id])?;
    Ok(())
}

pub fn create_playlist(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO playlists(name) VALUES(?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM playlists WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id=p.id), p.cover_path
         FROM playlists p ORDER BY p.name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Playlist {
            id: r.get(0)?,
            name: r.get(1)?,
            count: r.get(2)?,
            cover_path: r.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn set_playlist_cover(conn: &Connection, id: i64, cover: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET cover_path=?2 WHERE id=?1",
        params![id, cover],
    )?;
    Ok(())
}

pub fn rename_playlist(conn: &Connection, id: i64, name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET name=?2 WHERE id=?1",
        params![id, name],
    )?;
    Ok(())
}

pub fn add_to_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO playlist_tracks(playlist_id,track_id,position)
         VALUES(?1,?2,(SELECT COALESCE(MAX(position),0)+1 FROM playlist_tracks WHERE playlist_id=?1))",
        params![playlist_id, track_id],
    )?;
    Ok(())
}

pub fn remove_from_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id=?1 AND track_id=?2",
        params![playlist_id, track_id],
    )?;
    Ok(())
}

pub fn get_playlist(conn: &Connection, id: i64) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(
        "SELECT t.id,t.path,t.title,t.artist,t.album,t.duration,t.cover_path,t.source,t.url,t.liked
         FROM tracks t JOIN playlist_tracks pt ON pt.track_id=t.id
         WHERE pt.playlist_id=?1 ORDER BY pt.position",
    )?;
    let rows = stmt.query_map(params![id], row_to_track)?;
    rows.collect()
}

pub fn record_play(
    conn: &Connection,
    title: &str,
    artist: &str,
    source: &str,
    url: Option<&str>,
    played_at: i64,
    completed: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO history(title,artist,source,url,played_at,completed) VALUES(?1,?2,?3,?4,?5,?6)",
        params![title, artist, source, url, played_at, if completed { 1 } else { 0 }],
    )?;
    Ok(())
}

pub fn recent_history(conn: &Connection, since_ts: i64) -> Result<Vec<HistoryRow>> {
    let mut stmt = conn.prepare(
        "SELECT title,artist,source,url,played_at,completed FROM history
         WHERE played_at >= ?1 ORDER BY played_at DESC LIMIT 500",
    )?;
    let rows = stmt.query_map(params![since_ts], |r| {
        Ok(HistoryRow {
            title: r.get(0)?,
            artist: r.get(1)?,
            source: r.get(2)?,
            url: r.get(3)?,
            played_at: r.get(4)?,
            completed: r.get::<_, i64>(5)? != 0,
        })
    })?;
    rows.collect()
}

pub fn top_library_artists(conn: &Connection, limit: usize) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT artist, COUNT(*) as cnt FROM tracks
         WHERE artist != '' AND artist != 'Unknown Artist'
         GROUP BY artist ORDER BY cnt DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

pub fn toggle_like_track(conn: &Connection, id: i64) -> Result<bool> {
    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(liked, 0) FROM tracks WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let new_val = if current == 0 { 1 } else { 0 };
    conn.execute(
        "UPDATE tracks SET liked=?2 WHERE id=?1",
        params![id, new_val],
    )?;
    Ok(new_val != 0)
}

pub fn list_liked_tracks(conn: &Connection) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM tracks WHERE liked=1 ORDER BY id DESC"
    ))?;
    let rows = stmt.query_map([], row_to_track)?;
    rows.collect()
}

pub fn list_recent_library(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM tracks ORDER BY id DESC LIMIT ?1"
    ))?;
    let rows = stmt.query_map(params![limit as i64], row_to_track)?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        init(&c).unwrap();
        c
    }

    #[test]
    fn upsert_and_search() {
        let c = mem();
        upsert_track(&c, "a.mp3", "Hello", "Adele", "25", 10.0, None).unwrap();
        upsert_track(&c, "a.mp3", "Hello2", "Adele", "25", 10.0, None).unwrap();
        upsert_track(&c, "b.mp3", "Other", "Bob", "X", 5.0, None).unwrap();
        assert_eq!(list_tracks(&c, "").unwrap().len(), 2);
        let r = list_tracks(&c, "adele").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].title, "Hello2");
        assert_eq!(r[0].source, "local");
    }

    #[test]
    fn online_track_and_history() {
        let c = mem();
        let id = add_online_track(
            &c,
            "https://youtu.be/abc",
            ("Song", "Singer", "YouTube"),
            180.0,
            None,
            "youtube",
        )
        .unwrap();
        let id2 = add_online_track(
            &c,
            "https://youtu.be/abc",
            ("Song Updated", "Singer", "YouTube"),
            180.0,
            Some("cov.jpg"),
            "youtube",
        )
        .unwrap();
        assert_eq!(id, id2);
        set_track_file(&c, id, "C:/Music/song.m4a", None).unwrap();
        let t = get_track(&c, id).unwrap().unwrap();
        assert_eq!(t.path, "C:/Music/song.m4a");
        assert_eq!(t.url.as_deref(), Some("https://youtu.be/abc"));
        assert_eq!(t.cover_path.as_deref(), Some("cov.jpg"));

        record_play(
            &c,
            "Song Updated",
            "Singer",
            "youtube",
            t.url.as_deref(),
            1000,
            true,
        )
        .unwrap();
        let h = recent_history(&c, 500).unwrap();
        assert_eq!(h.len(), 1);
        assert!(h[0].completed);
    }

    #[test]
    fn playlist_flow() {
        let c = mem();
        upsert_track(&c, "a.mp3", "A", "x", "y", 1.0, None).unwrap();
        upsert_track(&c, "b.mp3", "B", "x", "y", 1.0, None).unwrap();
        let ts = list_tracks(&c, "").unwrap();
        let p = create_playlist(&c, "mix").unwrap();
        add_to_playlist(&c, p, ts[1].id).unwrap();
        add_to_playlist(&c, p, ts[0].id).unwrap();
        add_to_playlist(&c, p, ts[0].id).unwrap();
        let got = get_playlist(&c, p).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].title, "B");
        assert_eq!(list_playlists(&c).unwrap()[0].count, 2);
        remove_from_playlist(&c, p, ts[1].id).unwrap();
        assert_eq!(get_playlist(&c, p).unwrap().len(), 1);
        delete_playlist(&c, p).unwrap();
        assert!(list_playlists(&c).unwrap().is_empty());
    }

    #[test]
    fn playlist_cover_and_rename() {
        let c = mem();
        let p = create_playlist(&c, "old").unwrap();
        assert!(list_playlists(&c).unwrap()[0].cover_path.is_none());
        set_playlist_cover(&c, p, Some("x.png")).unwrap();
        rename_playlist(&c, p, "new").unwrap();
        let l = list_playlists(&c).unwrap();
        assert_eq!(l[0].name, "new");
        assert_eq!(l[0].cover_path.as_deref(), Some("x.png"));
        set_playlist_cover(&c, p, None).unwrap();
        assert!(list_playlists(&c).unwrap()[0].cover_path.is_none());
    }

    #[test]
    fn delete_track_cascades_playlist() {
        let c = mem();
        upsert_track(&c, "a.mp3", "A", "x", "y", 1.0, None).unwrap();
        let t = list_tracks(&c, "").unwrap()[0].id;
        let p = create_playlist(&c, "mix").unwrap();
        add_to_playlist(&c, p, t).unwrap();
        delete_track(&c, t).unwrap();
        assert!(list_tracks(&c, "").unwrap().is_empty());
        assert!(get_playlist(&c, p).unwrap().is_empty());
    }
}
