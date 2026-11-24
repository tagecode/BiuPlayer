use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

pub const SCHEMA_VERSION: i64 = 1;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub files_seen: u64,
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackRow {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub format: String,
}

pub struct Database {
    conn: Connection,
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
        let mut db = Self { conn };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn open_in_memory() -> Result<Self, LibraryError> {
        let conn = Connection::open_in_memory()?;
        let mut db = Self { conn };
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
        let applied: Option<i64> = self
            .conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [SCHEMA_VERSION],
                |row| row.get(0),
            )
            .optional()?;
        if applied.is_some() {
            return Ok(());
        }

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
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, datetime('now'))",
            [SCHEMA_VERSION],
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
        let started = Instant::now();
        let mut report = ScanReport {
            files_seen: 0,
            inserted: 0,
            updated: 0,
            skipped: 0,
            elapsed_ms: 0,
        };

        let tx = self.conn.transaction()?;
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
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    report.skipped += 1;
                    continue;
                }
            };
            report.files_seen += 1;
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let title = path
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_string();

            let changed = tx.execute(
                "INSERT INTO tracks(
                    folder_id, path, title, format, size_bytes, mtime, added_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))
                 ON CONFLICT(path) DO UPDATE SET
                    folder_id = excluded.folder_id,
                    title = excluded.title,
                    format = excluded.format,
                    size_bytes = excluded.size_bytes,
                    mtime = excluded.mtime,
                    updated_at = datetime('now')
                 WHERE tracks.size_bytes <> excluded.size_bytes OR tracks.mtime <> excluded.mtime",
                params![
                    folder_id,
                    path.display().to_string(),
                    title,
                    ext,
                    metadata.len() as i64,
                    mtime
                ],
            )?;
            if changed == 1 {
                report.inserted += 1;
            } else {
                report.skipped += 1;
            }
        }

        tx.execute(
            "UPDATE folders SET last_scan_at = datetime('now'), scan_state = 'complete' WHERE id = ?1",
            params![folder_id],
        )?;
        tx.commit()?;
        report.elapsed_ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<TrackRow>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.title, t.path, COALESCE(t.format, '')
             FROM tracks_fts f JOIN tracks t ON t.id = f.rowid
             WHERE tracks_fts MATCH ?1
             ORDER BY bm25(tracks_fts) LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
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

    pub fn track_count(&self) -> Result<u64, LibraryError> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?)
    }
}

pub fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

pub fn elapsed_since(start: Instant) -> Duration {
    start.elapsed()
}
