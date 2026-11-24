use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use std::collections::HashMap;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use pinyin::ToPinyin;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

pub const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database schema version {found} is newer than supported {supported}")]
    SchemaTooNew { found: i64, supported: i64 },
    #[error("database is corrupt: {0}")]
    Corrupt(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub files_seen: u64,
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
    pub elapsed_ms: u64,
}

/// Pause/cancel flags shared with a background scan thread.
#[derive(Debug, Default)]
pub struct ScanControl {
    paused: std::sync::atomic::AtomicBool,
    cancelled: std::sync::atomic::AtomicBool,
    pause_after: std::sync::atomic::AtomicU64,
}

impl ScanControl {
    pub fn pause(&self) {
        self.paused
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Pause automatically once this many files have been seen (tests/automation).
    pub fn pause_after(&self, files: u64) {
        self.pause_after
            .store(files, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Monotonic counters for a running scan.
#[derive(Debug, Default)]
pub struct ScanProgress {
    seen: std::sync::atomic::AtomicU64,
    inserted: std::sync::atomic::AtomicU64,
    skipped: std::sync::atomic::AtomicU64,
}

impl ScanProgress {
    pub fn snapshot(&self) -> (u64, u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        (
            self.seen.load(Relaxed),
            self.inserted.load(Relaxed),
            self.skipped.load(Relaxed),
        )
    }

    fn reset(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        self.seen.store(0, Relaxed);
        self.inserted.store(0, Relaxed);
        self.skipped.store(0, Relaxed);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScanCheckpoint {
    last_path: String,
    seen: u64,
    inserted: u64,
    skipped: u64,
}

#[derive(Debug)]
pub enum ScanOutcome {
    Completed(ScanReport),
    Paused(ScanReport),
    Cancelled(ScanReport),
}

impl ScanOutcome {
    pub fn report(&self) -> &ScanReport {
        match self {
            Self::Completed(report) | Self::Paused(report) | Self::Cancelled(report) => report,
        }
    }
}

#[derive(Debug, Default)]
struct TrackTags {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    track_no: Option<u32>,
    disc_no: Option<u32>,
    genre: Option<String>,
    year: Option<i32>,
    duration_ms: Option<i64>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    bitrate: Option<u32>,
    cover: Option<Vec<u8>>,
    cover_mime: Option<String>,
}

fn pinyin_parts(input: &str) -> (String, String, String) {
    let mut syllables: Vec<String> = Vec::new();
    let mut initials = String::new();
    let mut compact = String::new();
    for ch in input.chars() {
        if let Some(pinyin) = ch.to_pinyin() {
            let plain = pinyin.plain();
            syllables.push(plain.to_string());
            initials.push_str(&plain[..1]);
            compact.push_str(plain);
        } else if ch.is_ascii_alphanumeric() {
            syllables.push(ch.to_string());
            compact.push(ch);
        }
    }
    let initials_spaced = initials
        .chars()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    (syllables.join(" "), initials_spaced, compact)
}

fn build_search_text(title: &str, artist: Option<&str>, album: Option<&str>) -> String {
    let mut text = String::new();
    for part in [Some(title), artist, album].into_iter().flatten() {
        if part.trim().is_empty() {
            continue;
        }
        text.push_str(part);
        text.push(' ');
        let (syllables, initials, compact) = pinyin_parts(part);
        let initials_compact = initials.replace(' ', "");
        for token in [syllables, initials, initials_compact, compact] {
            if !token.is_empty() {
                text.push_str(&token);
                text.push(' ');
            }
        }
    }
    text.trim_end().to_string()
}

fn fts_query(input: &str) -> String {
    let terms: Vec<String> = input
        .split_whitespace()
        .filter_map(|term| {
            let cleaned: String = term
                .chars()
                .filter(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '-')
                .collect();
            (!cleaned.is_empty()).then(|| format!("{cleaned}*"))
        })
        .collect();
    terms.join(" AND ")
}

fn read_tags(path: &Path) -> Option<TrackTags> {
    let tagged = lofty::read_from_path(path).ok()?;
    let properties = tagged.properties();
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    Some(TrackTags {
        title: tag
            .and_then(|tag| tag.title())
            .map(|value| value.to_string()),
        artist: tag
            .and_then(|tag| tag.artist())
            .map(|value| value.to_string()),
        album: tag
            .and_then(|tag| tag.album())
            .map(|value| value.to_string()),
        track_no: tag.and_then(|tag| tag.track()),
        disc_no: tag.and_then(|tag| tag.disk()),
        genre: tag
            .and_then(|tag| tag.genre())
            .map(|value| value.to_string()),
        year: tag.and_then(|tag| tag.year()).map(|value| value as i32),
        duration_ms: Some(properties.duration().as_millis() as i64),
        sample_rate: properties.sample_rate(),
        channels: properties.channels().map(u16::from),
        bitrate: properties.audio_bitrate(),
        cover: tag
            .and_then(|tag| tag.pictures().first())
            .map(|picture| picture.data().to_vec())
            .filter(|bytes| bytes.len() <= 8 * 1024 * 1024),
        cover_mime: tag
            .and_then(|tag| tag.pictures().first())
            .and_then(|picture| picture.mime_type())
            .map(|mime| mime.as_str().to_string()),
    })
}

fn artist_id(
    tx: &rusqlite::Transaction<'_>,
    cache: &mut HashMap<String, i64>,
    name: &str,
) -> Result<i64, LibraryError> {
    if let Some(id) = cache.get(name) {
        return Ok(*id);
    }
    tx.execute(
        "INSERT INTO artists(name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
        params![name],
    )?;
    let id = tx.query_row(
        "SELECT id FROM artists WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )?;
    cache.insert(name.to_string(), id);
    Ok(id)
}

fn album_id(
    tx: &rusqlite::Transaction<'_>,
    cache: &mut HashMap<(String, Option<i64>), i64>,
    title: &str,
    artist_id: Option<i64>,
    year: Option<i32>,
) -> Result<i64, LibraryError> {
    let key = (title.to_string(), artist_id);
    if let Some(id) = cache.get(&key) {
        return Ok(*id);
    }
    let existing: Option<i64> = tx
        .query_row(
            "SELECT id FROM albums WHERE title = ?1 AND artist_id IS ?2",
            params![title, artist_id],
            |row| row.get(0),
        )
        .optional()?;
    let id = match existing {
        Some(id) => id,
        None => {
            tx.execute(
                "INSERT INTO albums(title, artist_id, year) VALUES (?1, ?2, ?3)",
                params![title, artist_id, year],
            )?;
            tx.last_insert_rowid()
        }
    };
    if year.is_some() {
        tx.execute(
            "UPDATE albums SET year = COALESCE(year, ?2) WHERE id = ?1",
            params![id, year],
        )?;
    }
    cache.insert(key, id);
    Ok(id)
}

#[derive(Debug, Clone)]
pub struct TrackRow {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct TrackListItem {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub format: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct PlaylistSummary {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
}

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub position: i64,
    pub track: TrackListItem,
}

#[derive(Debug, Clone)]
pub struct AlbumSummary {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub year: Option<i64>,
    pub track_count: i64,
}

#[derive(Debug, Clone)]
pub struct ArtistSummary {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
}

pub struct Database {
    conn: Connection,
    cover_dir: Option<std::path::PathBuf>,
}

/// Path of the pre-migration backup (`library.db.bak`).
pub fn backup_path(path: &Path) -> std::path::PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(".bak");
    std::path::PathBuf::from(value)
}

/// Inspect an existing database before opening it for migration.
///
/// - New/empty files need no backup.
/// - A database without `schema_migrations` is treated as legacy and copied.
/// - A database newer than this binary is rejected instead of downgraded.
fn prepare_existing(path: &Path) -> Result<(), LibraryError> {
    if !path.exists() || std::fs::metadata(path)?.len() == 0 {
        return Ok(());
    }

    let conn = Connection::open(path)?;
    let check: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if check != "ok" {
        return Err(LibraryError::Corrupt(check));
    }
    let has_migrations: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')",
        [],
        |row| row.get(0),
    )?;
    let max_version: i64 = if has_migrations {
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?
    } else {
        0
    };
    drop(conn);

    if max_version > SCHEMA_VERSION {
        return Err(LibraryError::SchemaTooNew {
            found: max_version,
            supported: SCHEMA_VERSION,
        });
    }
    if max_version < SCHEMA_VERSION {
        std::fs::copy(path, backup_path(path))?;
    }
    Ok(())
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LibraryError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        prepare_existing(path)?;
        let conn = Connection::open(path)?;
        let mut db = Self {
            conn,
            cover_dir: None,
        };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    pub fn set_cover_dir(&mut self, dir: impl AsRef<Path>) {
        self.cover_dir = Some(dir.as_ref().to_path_buf());
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn open_in_memory() -> Result<Self, LibraryError> {
        let conn = Connection::open_in_memory()?;
        let mut db = Self {
            conn,
            cover_dir: None,
        };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    fn configure(&self) -> Result<(), LibraryError> {
        self.conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;
             PRAGMA temp_store=MEMORY;",
        )?;
        Ok(())
    }

    fn migrate(&mut self) -> Result<(), LibraryError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )?;
        let current: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if current < 1 {
            self.apply_v1()?;
        }
        if current < 2 {
            self.apply_v2()?;
        }
        Ok(())
    }

    fn apply_v1(&mut self) -> Result<(), LibraryError> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            "CREATE TABLE folders (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                added_at TEXT NOT NULL,
                last_scan_at TEXT,
                scan_state TEXT
            );
            CREATE TABLE artists (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                sort_name TEXT
            );
            CREATE TABLE albums (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id INTEGER REFERENCES artists(id) ON DELETE SET NULL,
                year INTEGER,
                cover_path TEXT,
                UNIQUE(title, artist_id)
            );
            CREATE TABLE tracks (
                id INTEGER PRIMARY KEY,
                folder_id INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
                path TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL DEFAULT '',
                artist_id INTEGER REFERENCES artists(id) ON DELETE SET NULL,
                album_id INTEGER REFERENCES albums(id) ON DELETE SET NULL,
                track_no INTEGER,
                disc_no INTEGER,
                genre TEXT,
                year INTEGER,
                duration_ms INTEGER,
                sample_rate INTEGER,
                channels INTEGER,
                bitrate INTEGER,
                format TEXT,
                size_bytes INTEGER,
                mtime INTEGER,
                added_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                missing INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE playlists (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE playlist_items (
                playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
                track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, position)
            );
            CREATE TABLE lyrics (
                track_id INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                offset_ms INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE play_history (
                id INTEGER PRIMARY KEY,
                track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
                played_at TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX idx_tracks_folder ON tracks(folder_id);
            CREATE INDEX idx_tracks_album ON tracks(album_id);
            CREATE INDEX idx_tracks_artist ON tracks(artist_id);
            CREATE INDEX idx_tracks_mtime ON tracks(mtime);
            CREATE VIRTUAL TABLE tracks_fts USING fts5(
                title, path,
                content='tracks', content_rowid='id',
                tokenize='unicode61'
            );
            CREATE TRIGGER tracks_ai AFTER INSERT ON tracks BEGIN
                INSERT INTO tracks_fts(rowid, title, path) VALUES (new.id, new.title, new.path);
            END;
            CREATE TRIGGER tracks_ad AFTER DELETE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, path)
                VALUES ('delete', old.id, old.title, old.path);
            END;
            CREATE TRIGGER tracks_au AFTER UPDATE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, path)
                VALUES ('delete', old.id, old.title, old.path);
                INSERT INTO tracks_fts(rowid, title, path) VALUES (new.id, new.title, new.path);
            END;",
        )?;
        tx.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (1, datetime('now'))",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn apply_v2(&mut self) -> Result<(), LibraryError> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            "ALTER TABLE tracks ADD COLUMN search_text TEXT NOT NULL DEFAULT '';
            DROP TRIGGER tracks_ai;
            DROP TRIGGER tracks_ad;
            DROP TRIGGER tracks_au;
            DROP TABLE tracks_fts;
            CREATE VIRTUAL TABLE tracks_fts USING fts5(
                title, search_text, path,
                content='tracks', content_rowid='id',
                tokenize='unicode61'
            );
            INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
            CREATE TRIGGER tracks_ai AFTER INSERT ON tracks BEGIN
                INSERT INTO tracks_fts(rowid, title, search_text, path)
                VALUES (new.id, new.title, new.search_text, new.path);
            END;
            CREATE TRIGGER tracks_ad AFTER DELETE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, search_text, path)
                VALUES ('delete', old.id, old.title, old.search_text, old.path);
            END;
            CREATE TRIGGER tracks_au AFTER UPDATE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, search_text, path)
                VALUES ('delete', old.id, old.title, old.search_text, old.path);
                INSERT INTO tracks_fts(rowid, title, search_text, path)
                VALUES (new.id, new.title, new.search_text, new.path);
            END;",
        )?;
        tx.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (2, datetime('now'))",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn add_folder(&self, path: &Path) -> Result<i64, LibraryError> {
        let path = path.canonicalize()?;
        self.conn.execute(
            "INSERT INTO folders(path, added_at) VALUES (?1, datetime('now'))
             ON CONFLICT(path) DO NOTHING",
            params![path.display().to_string()],
        )?;
        let id = self.conn.query_row(
            "SELECT id FROM folders WHERE path = ?1",
            params![path.display().to_string()],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn scan_folder(&mut self, folder_id: i64, root: &Path) -> Result<ScanReport, LibraryError> {
        let control = ScanControl::default();
        let progress = ScanProgress::default();
        match self.scan_folder_controlled(folder_id, root, &control, &progress)? {
            ScanOutcome::Completed(report) => Ok(report),
            ScanOutcome::Paused(_) | ScanOutcome::Cancelled(_) => {
                unreachable!("a scan without control flags cannot pause or cancel")
            }
        }
    }

    fn scan_checkpoint(&self, folder_id: i64) -> Result<Option<ScanCheckpoint>, LibraryError> {
        let state: Option<String> = self
            .conn
            .query_row(
                "SELECT scan_state FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        Ok(state.and_then(|raw| serde_json::from_str::<ScanCheckpoint>(&raw).ok()))
    }

    fn store_checkpoint(
        &self,
        folder_id: i64,
        last_path: &str,
        report: &ScanReport,
    ) -> Result<(), LibraryError> {
        let checkpoint = ScanCheckpoint {
            last_path: last_path.to_string(),
            seen: report.files_seen,
            inserted: report.inserted,
            skipped: report.skipped,
        };
        let raw = serde_json::to_string(&checkpoint)
            .map_err(|err| LibraryError::Io(std::io::Error::other(err)))?;
        self.conn.execute(
            "UPDATE folders SET scan_state = ?1 WHERE id = ?2",
            params![raw, folder_id],
        )?;
        Ok(())
    }

    /// Scan a folder while honoring pause/cancel and persisting a resume checkpoint.
    ///
    /// The checkpoint stores the last fully processed path; resuming walks the tree
    /// again and skips entries up to (and including) that path. Unchanged files are
    /// skipped before tags are read by comparing size and mtime.
    pub fn scan_folder_controlled(
        &mut self,
        folder_id: i64,
        root: &Path,
        control: &ScanControl,
        progress: &ScanProgress,
    ) -> Result<ScanOutcome, LibraryError> {
        let started = Instant::now();
        progress.reset();
        let mut report = ScanReport {
            files_seen: 0,
            inserted: 0,
            updated: 0,
            skipped: 0,
            elapsed_ms: 0,
        };

        let resume_from = self.scan_checkpoint(folder_id)?.map(|c| c.last_path);
        let mut resuming = resume_from.is_some();
        let mut last_path: Option<String> = None;
        let mut since_checkpoint = 0u64;
        let mut artist_cache: HashMap<String, i64> = HashMap::new();
        let mut album_cache: HashMap<(String, Option<i64>), i64> = HashMap::new();

        for entry in WalkDir::new(root).follow_links(false).into_iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    report.skipped += 1;
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(
                ext.as_str(),
                "mp3"
                    | "flac"
                    | "wav"
                    | "wave"
                    | "m4a"
                    | "mp4"
                    | "aac"
                    | "ogg"
                    | "oga"
                    | "aif"
                    | "aiff"
                    | "aifc"
            ) {
                continue;
            }
            let path_string = path.display().to_string();
            if resuming {
                if resume_from.as_deref() == Some(path_string.as_str()) {
                    resuming = false;
                }
                continue;
            }
            let pause_after = control
                .pause_after
                .load(std::sync::atomic::Ordering::Relaxed);
            if pause_after > 0
                && progress.seen.load(std::sync::atomic::Ordering::Relaxed) >= pause_after
            {
                control.pause();
            }
            if control.is_cancelled() {
                report.elapsed_ms = started.elapsed().as_millis() as u64;
                if let Some(last) = last_path.as_deref() {
                    self.store_checkpoint(folder_id, last, &report)?;
                }
                return Ok(ScanOutcome::Cancelled(report));
            }
            if control.is_paused() {
                report.elapsed_ms = started.elapsed().as_millis() as u64;
                if let Some(last) = last_path.as_deref() {
                    self.store_checkpoint(folder_id, last, &report)?;
                }
                return Ok(ScanOutcome::Paused(report));
            }

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    report.skipped += 1;
                    progress
                        .skipped
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    continue;
                }
            };
            report.files_seen += 1;
            progress
                .seen
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let size = metadata.len() as i64;

            let tx = self.conn.transaction()?;
            let existing: Option<(i64, i64, String)> = tx
                .query_row(
                    "SELECT size_bytes, mtime, search_text FROM tracks WHERE path = ?1",
                    params![path_string],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;
            let unchanged = existing
                .as_ref()
                .is_some_and(|(stored_size, stored_mtime, text)| {
                    *stored_size == size && *stored_mtime == mtime && !text.is_empty()
                });
            if unchanged {
                tx.commit()?;
                report.skipped += 1;
                progress
                    .skipped
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                last_path = Some(path_string);
                since_checkpoint += 1;
                if since_checkpoint >= 50 {
                    if let Some(last) = last_path.as_deref() {
                        self.store_checkpoint(folder_id, last, &report)?;
                    }
                    since_checkpoint = 0;
                }
                continue;
            }

            let tags = read_tags(path);
            let title = tags
                .as_ref()
                .and_then(|tags| tags.title.clone())
                .or_else(|| {
                    path.file_stem()
                        .and_then(|v| v.to_str())
                        .map(ToString::to_string)
                })
                .unwrap_or_default();
            let artist_name = tags
                .as_ref()
                .and_then(|tags| tags.artist.as_deref())
                .filter(|name| !name.trim().is_empty());
            let album_name = tags
                .as_ref()
                .and_then(|tags| tags.album.as_deref())
                .filter(|name| !name.trim().is_empty());
            let search_text = build_search_text(&title, artist_name, album_name);

            let artist_id = match artist_name {
                Some(name) => Some(artist_id(&tx, &mut artist_cache, name)?),
                None => None,
            };
            let album_id = match album_name {
                Some(title) => Some(album_id(
                    &tx,
                    &mut album_cache,
                    title,
                    artist_id,
                    tags.as_ref().and_then(|tags| tags.year),
                )?),
                None => None,
            };

            if let (Some(album), Some(cover_dir), Some(tags)) =
                (album_id, self.cover_dir.as_deref(), tags.as_ref())
            {
                let existing: Option<String> = tx
                    .query_row(
                        "SELECT COALESCE(cover_path, '') FROM albums WHERE id = ?1",
                        params![album],
                        |row| row.get(0),
                    )
                    .optional()?;
                if existing.as_deref().unwrap_or("").is_empty() {
                    if let Some(bytes) = tags.cover.as_deref() {
                        let ext = match tags.cover_mime.as_deref() {
                            Some("image/png") => "png",
                            Some("image/gif") => "gif",
                            Some("image/bmp") => "bmp",
                            Some("image/tiff") => "tiff",
                            _ => "jpg",
                        };
                        let hash = blake3::hash(bytes).to_hex().to_string();
                        std::fs::create_dir_all(cover_dir)?;
                        let file = cover_dir.join(format!("{hash}.{ext}"));
                        if !file.exists() {
                            std::fs::write(&file, bytes)?;
                        }
                        tx.execute(
                            "UPDATE albums SET cover_path = ?1 WHERE id = ?2",
                            params![file.to_string_lossy(), album],
                        )?;
                    }
                }
            }

            tx.execute(
                "INSERT INTO tracks(
                    folder_id, path, title, search_text, artist_id, album_id, track_no, disc_no,
                    genre, year, duration_ms, sample_rate, channels, bitrate,
                    format, size_bytes, mtime, added_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                    datetime('now'), datetime('now'))
                 ON CONFLICT(path) DO UPDATE SET
                    folder_id = excluded.folder_id,
                    title = excluded.title,
                    search_text = excluded.search_text,
                    artist_id = excluded.artist_id,
                    album_id = excluded.album_id,
                    track_no = excluded.track_no,
                    disc_no = excluded.disc_no,
                    genre = excluded.genre,
                    year = excluded.year,
                    duration_ms = excluded.duration_ms,
                    sample_rate = excluded.sample_rate,
                    channels = excluded.channels,
                    bitrate = excluded.bitrate,
                    format = excluded.format,
                    size_bytes = excluded.size_bytes,
                    mtime = excluded.mtime,
                    updated_at = datetime('now')",
                params![
                    folder_id,
                    path_string,
                    title,
                    search_text,
                    artist_id,
                    album_id,
                    tags.as_ref().and_then(|tags| tags.track_no),
                    tags.as_ref().and_then(|tags| tags.disc_no),
                    tags.as_ref().and_then(|tags| tags.genre.clone()),
                    tags.as_ref().and_then(|tags| tags.year),
                    tags.as_ref().and_then(|tags| tags.duration_ms),
                    tags.as_ref().and_then(|tags| tags.sample_rate),
                    tags.as_ref().and_then(|tags| tags.channels),
                    tags.as_ref().and_then(|tags| tags.bitrate),
                    ext,
                    size,
                    mtime
                ],
            )?;
            tx.commit()?;
            if existing.is_some() {
                report.updated += 1;
                progress
                    .inserted
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                report.inserted += 1;
                progress
                    .inserted
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            last_path = Some(path_string);
            since_checkpoint += 1;
            if since_checkpoint >= 50 {
                if let Some(last) = last_path.as_deref() {
                    self.store_checkpoint(folder_id, last, &report)?;
                }
                since_checkpoint = 0;
            }
        }

        self.conn.execute(
            "UPDATE folders SET last_scan_at = datetime('now'), scan_state = 'complete' WHERE id = ?1",
            params![folder_id],
        )?;
        report.elapsed_ms = started.elapsed().as_millis() as u64;
        Ok(ScanOutcome::Completed(report))
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<TrackRow>, LibraryError> {
        let match_query = fts_query(query);
        if match_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.title, t.path, COALESCE(t.format, '')
             FROM tracks_fts f JOIN tracks t ON t.id = f.rowid
             WHERE tracks_fts MATCH ?1
             ORDER BY bm25(tracks_fts) LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![match_query, limit as i64], |row| {
            Ok(TrackRow {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                format: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn list_tracks(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TrackListItem>, LibraryError> {
        self.list_tracks_sorted(limit, offset, "title", false)
    }

    pub fn list_tracks_sorted(
        &self,
        limit: usize,
        offset: usize,
        sort: &str,
        descending: bool,
    ) -> Result<Vec<TrackListItem>, LibraryError> {
        let order = match sort {
            "artist" => "ar.name COLLATE NOCASE",
            "album" => "al.title COLLATE NOCASE",
            "duration" => "t.duration_ms",
            "added" => "t.added_at",
            "path" => "t.path COLLATE NOCASE",
            _ => "t.title COLLATE NOCASE",
        };
        let direction = if descending { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_ms, t.format, t.path
             FROM tracks t
             LEFT JOIN artists ar ON ar.id = t.artist_id
             LEFT JOIN albums al ON al.id = t.album_id
             WHERE t.missing = 0
             ORDER BY {order} {direction}, t.id {direction}
             LIMIT ?1 OFFSET ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(TrackListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_ms: row.get(4)?,
                format: row.get(5)?,
                path: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn track_count(&self) -> Result<u64, LibraryError> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?)
    }
    pub fn list_playlists(&self) -> Result<Vec<PlaylistSummary>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, COUNT(pi.track_id)
             FROM playlists p
             LEFT JOIN playlist_items pi ON pi.playlist_id = p.id
             GROUP BY p.id
             ORDER BY p.name COLLATE NOCASE, p.id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PlaylistSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                track_count: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn create_playlist(&self, name: &str) -> Result<i64, LibraryError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LibraryError::InvalidArgument(
                "playlist name is empty".into(),
            ));
        }
        self.conn.execute(
            "INSERT INTO playlists (name, created_at, updated_at)
             VALUES (?1, datetime('now'), datetime('now'))",
            params![name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<(), LibraryError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LibraryError::InvalidArgument(
                "playlist name is empty".into(),
            ));
        }
        let changed = self.conn.execute(
            "UPDATE playlists SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, id],
        )?;
        if changed == 0 {
            return Err(LibraryError::InvalidArgument("playlist not found".into()));
        }
        Ok(())
    }

    pub fn delete_playlist(&self, id: i64) -> Result<(), LibraryError> {
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn playlist_items(&self, id: i64) -> Result<Vec<PlaylistItem>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT pi.position, t.id, t.title, ar.name, al.title,
                    t.duration_ms, t.format, t.path
             FROM playlist_items pi
             JOIN tracks t ON t.id = pi.track_id
             LEFT JOIN artists ar ON ar.id = t.artist_id
             LEFT JOIN albums al ON al.id = t.album_id
             WHERE pi.playlist_id = ?1
             ORDER BY pi.position",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            Ok(PlaylistItem {
                position: row.get(0)?,
                track: TrackListItem {
                    id: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    album: row.get(4)?,
                    duration_ms: row.get(5)?,
                    format: row.get(6)?,
                    path: row.get(7)?,
                },
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn add_tracks_to_playlist(
        &self,
        id: i64,
        track_ids: &[i64],
    ) -> Result<usize, LibraryError> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM playlists WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(LibraryError::InvalidArgument("playlist not found".into()));
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut inserted = 0usize;
        for track_id in track_ids {
            let changed = tx.execute(
                "INSERT INTO playlist_items (playlist_id, track_id, position)
                 SELECT ?1, ?2, COALESCE((SELECT MAX(position) + 1
                                          FROM playlist_items WHERE playlist_id = ?1), 0)
                 WHERE EXISTS(SELECT 1 FROM tracks WHERE id = ?2)
                   AND NOT EXISTS(SELECT 1 FROM playlist_items
                                  WHERE playlist_id = ?1 AND track_id = ?2)",
                params![id, track_id],
            )?;
            inserted += changed;
        }
        tx.execute(
            "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        tx.commit()?;
        Ok(inserted)
    }

    pub fn remove_playlist_item(&self, id: i64, position: i64) -> Result<(), LibraryError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut stmt = tx.prepare(
            "SELECT track_id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position",
        )?;
        let mut ids: Vec<i64> = stmt
            .query_map(params![id], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        drop(stmt);
        if position < 0 || position >= ids.len() as i64 {
            return Err(LibraryError::InvalidArgument(
                "position out of range".into(),
            ));
        }
        ids.remove(position as usize);
        tx.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1",
            params![id],
        )?;
        for (index, track_id) in ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO playlist_items (playlist_id, track_id, position)
                 VALUES (?1, ?2, ?3)",
                params![id, track_id, index as i64],
            )?;
        }
        tx.execute(
            "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn move_playlist_item(&self, id: i64, from: i64, to: i64) -> Result<(), LibraryError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut stmt = tx.prepare(
            "SELECT track_id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position",
        )?;
        let ids: Vec<i64> = stmt
            .query_map(params![id], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        drop(stmt);
        if from < 0 || from >= ids.len() as i64 || to < 0 || to >= ids.len() as i64 {
            return Err(LibraryError::InvalidArgument(
                "position out of range".into(),
            ));
        }
        let mut ids = ids;
        let value = ids.remove(from as usize);
        ids.insert(to as usize, value);
        tx.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1",
            params![id],
        )?;
        for (position, track_id) in ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO playlist_items (playlist_id, track_id, position)
                 VALUES (?1, ?2, ?3)",
                params![id, track_id, position as i64],
            )?;
        }
        tx.execute(
            "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn list_albums(&self) -> Result<Vec<AlbumSummary>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT al.id, al.title, ar.name, al.year, COUNT(t.id)
             FROM albums al
             LEFT JOIN artists ar ON ar.id = al.artist_id
             LEFT JOIN tracks t ON t.album_id = al.id AND t.missing = 0
             GROUP BY al.id
             ORDER BY al.title COLLATE NOCASE, al.id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AlbumSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                year: row.get(3)?,
                track_count: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn album_cover_path(&self, id: i64) -> Result<Option<String>, LibraryError> {
        let path: Option<String> = self
            .conn
            .query_row(
                "SELECT COALESCE(cover_path, '') FROM albums WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(path.filter(|value| !value.is_empty()))
    }

    pub fn album_tracks(&self, id: i64) -> Result<Vec<TrackListItem>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_ms, t.format, t.path
             FROM tracks t
             LEFT JOIN artists ar ON ar.id = t.artist_id
             LEFT JOIN albums al ON al.id = t.album_id
             WHERE t.album_id = ?1 AND t.missing = 0
             ORDER BY t.track_no, t.title COLLATE NOCASE, t.id",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            Ok(TrackListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_ms: row.get(4)?,
                format: row.get(5)?,
                path: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn list_artists(&self) -> Result<Vec<ArtistSummary>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT ar.id, ar.name, COUNT(t.id)
             FROM artists ar
             LEFT JOIN tracks t ON t.artist_id = ar.id AND t.missing = 0
             GROUP BY ar.id
             ORDER BY ar.name COLLATE NOCASE, ar.id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ArtistSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                track_count: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn artist_tracks(&self, id: i64) -> Result<Vec<TrackListItem>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_ms, t.format, t.path
             FROM tracks t
             LEFT JOIN artists ar ON ar.id = t.artist_id
             LEFT JOIN albums al ON al.id = t.album_id
             WHERE t.artist_id = ?1 AND t.missing = 0
             ORDER BY al.title COLLATE NOCASE, t.track_no, t.title COLLATE NOCASE, t.id",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            Ok(TrackListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_ms: row.get(4)?,
                format: row.get(5)?,
                path: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

pub fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

pub fn elapsed_since(start: Instant) -> Duration {
    start.elapsed()
}
