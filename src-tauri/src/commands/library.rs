use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use base64::Engine;
use biu_ipc::{
    AlbumDto, ArtistDto, CommandError, FolderDto, GenreDto, LibraryScanDto, LibraryScanStatusDto,
    LibraryTrackDto, PlaylistDto, PlaylistItemDto, YearDto,
};
use biu_library::{Database, ScanControl, ScanOutcome, ScanProgress, ScanReport, TrackListItem};
use tauri::State;

use crate::commands::settings::SettingsState;

#[derive(Default)]
pub struct ScanSession {
    pub status: LibraryScanStatusDto,
    control: Option<Arc<ScanControl>>,
    progress: Option<Arc<ScanProgress>>,
}

pub struct LibraryState {
    pub db: Arc<Mutex<Option<Database>>>,
    pub database_file: PathBuf,
    pub scan: Arc<Mutex<ScanSession>>,
}

impl LibraryState {
    pub fn new(db: Option<Database>, database_file: PathBuf) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            database_file,
            scan: Arc::new(Mutex::new(ScanSession::default())),
        }
    }
}

fn lock_db(state: &LibraryState) -> std::sync::MutexGuard<'_, Option<Database>> {
    state
        .db
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_scan(state: &LibraryState) -> std::sync::MutexGuard<'_, ScanSession> {
    state
        .scan
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn scan_dto(report: ScanReport, total: f64) -> LibraryScanDto {
    LibraryScanDto {
        files_seen: report.files_seen as f64,
        inserted: report.inserted as f64,
        updated: report.updated as f64,
        skipped: report.skipped as f64,
        elapsed_ms: report.elapsed_ms as f64,
        total_tracks: total,
    }
}

fn canonical_folder(path: &str) -> Result<PathBuf, CommandError> {
    let root = Path::new(path);
    if !root.is_dir() {
        return Err(CommandError::invalid_argument(
            "folder does not exist or is not a directory",
        ));
    }
    root.canonicalize()
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn library_add_folder(
    path: String,
    state: State<'_, LibraryState>,
) -> Result<LibraryScanDto, CommandError> {
    let root = canonical_folder(&path)?;

    let mut guard = lock_db(&state);
    let database = guard
        .as_mut()
        .ok_or_else(|| CommandError::internal("library database is not open"))?;
    let folder_id = database
        .add_folder(&root)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    let report = database
        .scan_folder(folder_id, &root)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    let total = database
        .track_count()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(scan_dto(report, total as f64))
}

fn begin_scan(
    state: &LibraryState,
    root: PathBuf,
    excludes: Vec<String>,
) -> Result<LibraryScanStatusDto, CommandError> {
    let mut session = lock_scan(state);
    if session.status.state == "scanning" || session.status.state == "cancelling" {
        return Err(CommandError::invalid_argument("a scan is already running"));
    }
    if session.control.is_some() {
        return Err(CommandError::invalid_argument(
            "the previous scan is still stopping",
        ));
    }

    let control = Arc::new(ScanControl::default());
    let progress = Arc::new(ScanProgress::default());
    let status = LibraryScanStatusDto {
        state: "scanning".to_string(),
        root: Some(root.display().to_string()),
        files_seen: 0.0,
        inserted: 0.0,
        skipped: 0.0,
        total_tracks: session.status.total_tracks,
        error: None,
    };
    session.status = status.clone();
    session.control = Some(control.clone());
    session.progress = Some(progress.clone());
    drop(session);

    let database_file = state.database_file.clone();
    let session = state.scan.clone();
    std::thread::spawn(move || {
        let result = (|| {
            let mut database = Database::open(&database_file)?;
            if let Some(parent) = database_file.parent() {
                database.set_cover_dir(parent.join("covers"));
            }
            let folder_id = database.add_folder(&root)?;
            database
                .scan_folder_controlled_filtered(folder_id, &root, &control, &progress, &excludes)
        })();

        let mut session = session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match result {
            Ok(ScanOutcome::Completed(report)) => {
                session.status.state = "completed".to_string();
                session.status.files_seen = report.files_seen as f64;
                session.status.inserted = report.inserted as f64;
                session.status.skipped = report.skipped as f64;
                session.status.total_tracks += report.inserted as f64;
            }
            Ok(ScanOutcome::Paused(report)) => {
                session.status.state = "paused".to_string();
                session.status.files_seen = report.files_seen as f64;
                session.status.inserted = report.inserted as f64;
                session.status.skipped = report.skipped as f64;
            }
            Ok(ScanOutcome::Cancelled(report)) => {
                session.status.state = "cancelled".to_string();
                session.status.files_seen = report.files_seen as f64;
                session.status.inserted = report.inserted as f64;
                session.status.skipped = report.skipped as f64;
            }
            Err(err) => {
                session.status.state = "error".to_string();
                session.status.error = Some(err.to_string());
            }
        }
        session.control = None;
        session.progress = None;
    });

    Ok(status)
}

#[tauri::command]
#[specta::specta]
pub fn library_scan_start(
    path: String,
    state: State<'_, LibraryState>,
    settings: State<'_, SettingsState>,
) -> Result<LibraryScanStatusDto, CommandError> {
    let root = canonical_folder(&path)?;
    let excludes = settings
        .settings
        .lock()
        .map(|settings| settings.library.exclude_globs.clone())
        .unwrap_or_default();
    begin_scan(&state, root, excludes)
}

#[tauri::command]
#[specta::specta]
pub fn library_remove_folder(id: f64, state: State<'_, LibraryState>) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    if !database
        .remove_folder(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))?
    {
        return Err(CommandError::invalid_argument("folder not found"));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn library_scan_status(state: State<'_, LibraryState>) -> LibraryScanStatusDto {
    let mut session = lock_scan(&state);
    if session.status.state == "scanning" {
        if let Some(progress) = session.progress.as_ref() {
            let (seen, inserted, skipped) = progress.snapshot();
            session.status.files_seen = seen as f64;
            session.status.inserted = inserted as f64;
            session.status.skipped = skipped as f64;
        }
    }
    session.status.clone()
}

#[tauri::command]
#[specta::specta]
pub fn library_scan_pause(
    state: State<'_, LibraryState>,
) -> Result<LibraryScanStatusDto, CommandError> {
    let mut session = lock_scan(&state);
    if session.status.state != "scanning" {
        return Err(CommandError::invalid_argument("no scan is running"));
    }
    if let Some(control) = session.control.as_ref() {
        control.pause();
    }
    session.status.state = "paused".to_string();
    Ok(session.status.clone())
}

#[tauri::command]
#[specta::specta]
pub fn library_scan_resume(
    state: State<'_, LibraryState>,
    settings: State<'_, SettingsState>,
) -> Result<LibraryScanStatusDto, CommandError> {
    let root = {
        let session = lock_scan(&state);
        if session.status.state != "paused" && session.status.state != "cancelled" {
            return Err(CommandError::invalid_argument("no paused scan to resume"));
        }
        session
            .status
            .root
            .clone()
            .ok_or_else(|| CommandError::internal("scan root is missing"))?
    };
    let excludes = settings
        .settings
        .lock()
        .map(|settings| settings.library.exclude_globs.clone())
        .unwrap_or_default();
    begin_scan(&state, PathBuf::from(root), excludes)
}

#[tauri::command]
#[specta::specta]
pub fn library_scan_cancel(
    state: State<'_, LibraryState>,
) -> Result<LibraryScanStatusDto, CommandError> {
    let mut session = lock_scan(&state);
    if session.status.state != "scanning" && session.status.state != "paused" {
        return Err(CommandError::invalid_argument("no scan to cancel"));
    }
    if let Some(control) = session.control.as_ref() {
        control.cancel();
    }
    session.status.state = "cancelled".to_string();
    Ok(session.status.clone())
}

fn track_dto(track: TrackListItem) -> LibraryTrackDto {
    LibraryTrackDto {
        id: track.id as f64,
        title: track.title,
        artist: track.artist,
        album: track.album,
        duration_ms: track.duration_ms.map(|value| value as f64),
        format: track.format,
        path: track.path,
        added_at: track.added_at,
    }
}

#[tauri::command]
#[specta::specta]
pub fn library_tracks(
    limit: u32,
    offset: u32,
    sort: Option<String>,
    descending: Option<bool>,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let tracks = database
        .list_tracks_sorted(
            limit.min(1000) as usize,
            offset as usize,
            sort.as_deref().unwrap_or("title"),
            descending.unwrap_or(false),
        )
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(tracks.into_iter().map(track_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_track_count(state: State<'_, LibraryState>) -> f64 {
    let guard = lock_db(&state);
    guard
        .as_ref()
        .and_then(|database| database.track_count().ok())
        .unwrap_or(0) as f64
}

#[tauri::command]
#[specta::specta]
pub fn library_move_track(
    track_id: f64,
    before_id: Option<f64>,
    after_id: Option<f64>,
    state: tauri::State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let mut guard = state.db.lock().unwrap();
    let Some(db) = guard.as_mut() else {
        return Err(CommandError::internal("library not initialized"));
    };
    db.move_track(
        track_id as i64,
        before_id.map(|value| value as i64),
        after_id.map(|value| value as i64),
    )
    .map_err(|err| CommandError::internal(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn library_search(
    query: String,
    limit: u32,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let tracks = database
        .search(query, limit.min(200) as usize)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(tracks.into_iter().map(track_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub fn playlist_list(state: State<'_, LibraryState>) -> Result<Vec<PlaylistDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_playlists()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|row| PlaylistDto {
            id: row.id as f64,
            name: row.name,
            track_count: row.track_count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn playlist_create(name: String, state: State<'_, LibraryState>) -> Result<f64, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .create_playlist(&name)
        .map(|id| id as f64)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_rename(
    id: f64,
    name: String,
    state: State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .rename_playlist(id as i64, &name)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_delete(id: f64, state: State<'_, LibraryState>) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .delete_playlist(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_reorder_all(
    ids: Vec<f64>,
    state: State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    let ids: Vec<i64> = ids.into_iter().map(|id| id as i64).collect();
    database
        .reorder_playlists(&ids)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_items(
    id: f64,
    state: State<'_, LibraryState>,
) -> Result<Vec<PlaylistItemDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .playlist_items(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|row| PlaylistItemDto {
            position: row.position as f64,
            track: track_dto(row.track),
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn playlist_add_tracks(
    id: f64,
    track_ids: Vec<f64>,
    state: State<'_, LibraryState>,
) -> Result<f64, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    let ids: Vec<i64> = track_ids.iter().map(|value| *value as i64).collect();
    database
        .add_tracks_to_playlist(id as i64, &ids)
        .map(|count| count as f64)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_copy(
    id: f64,
    name: String,
    state: State<'_, LibraryState>,
) -> Result<f64, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .copy_playlist(id as i64, &name)
        .map(|value| value as f64)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_remove_items(
    id: f64,
    positions: Vec<f64>,
    state: State<'_, LibraryState>,
) -> Result<f64, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    let values: Vec<i64> = positions.iter().map(|value| *value as i64).collect();
    database
        .remove_playlist_items(id as i64, &values)
        .map(|count| count as f64)
        .map_err(|err| CommandError::internal(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_reorder(
    id: f64,
    track_ids: Vec<f64>,
    state: State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    let values: Vec<i64> = track_ids.iter().map(|value| *value as i64).collect();
    database
        .reorder_playlist_items(id as i64, &values)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_add_paths(
    id: f64,
    paths: Vec<String>,
    state: State<'_, LibraryState>,
) -> Result<f64, CommandError> {
    let mut guard = lock_db(&state);
    let Some(database) = guard.as_mut() else {
        return Err(CommandError::internal("library database is not open"));
    };
    let playlist_id = id as i64;
    let mut roots: Vec<PathBuf> = Vec::new();
    for raw in &paths {
        let path = PathBuf::from(raw);
        if !path.exists() {
            continue;
        }
        let root = if path.is_dir() {
            path
        } else {
            match path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => continue,
            }
        };
        let root = root.canonicalize().unwrap_or(root);
        if !roots.iter().any(|entry| entry == &root) {
            roots.push(root);
        }
    }
    for root in &roots {
        let folder = database
            .add_folder(root)
            .map_err(|err| CommandError::invalid_argument(err.to_string()))?;
        database
            .scan_folder(folder, root)
            .map_err(|err| CommandError::internal(err.to_string()))?;
    }
    let mut track_ids: Vec<i64> = Vec::new();
    for raw in &paths {
        let path = PathBuf::from(raw);
        let path = path.canonicalize().unwrap_or(path);
        let value = path.to_string_lossy().to_string();
        if path.is_dir() {
            track_ids.extend(
                database
                    .track_ids_under(&value)
                    .map_err(|err| CommandError::internal(err.to_string()))?,
            );
        } else if let Some(track_id) = database
            .track_id_by_path(&value)
            .map_err(|err| CommandError::internal(err.to_string()))?
        {
            track_ids.push(track_id);
        }
    }
    track_ids.sort_unstable();
    track_ids.dedup();
    database
        .add_tracks_to_playlist(playlist_id, &track_ids)
        .map(|count| count as f64)
        .map_err(|err| CommandError::invalid_argument(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_remove_item(
    id: f64,
    position: f64,
    state: State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .remove_playlist_item(id as i64, position as i64)
        .map_err(|err| CommandError::internal(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn playlist_move_item(
    id: f64,
    from: f64,
    to: f64,
    state: State<'_, LibraryState>,
) -> Result<(), CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Err(CommandError::internal("library database is not open"));
    };
    database
        .move_playlist_item(id as i64, from as i64, to as i64)
        .map_err(|err| CommandError::internal(err.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn library_albums(state: State<'_, LibraryState>) -> Result<Vec<AlbumDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_albums()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|row| AlbumDto {
            id: row.id as f64,
            title: row.title,
            artist: row.artist,
            year: row.year.map(|value| value as f64),
            track_count: row.track_count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_album_tracks(
    id: f64,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .album_tracks(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows.into_iter().map(track_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_artists(state: State<'_, LibraryState>) -> Result<Vec<ArtistDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_artists()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|row| ArtistDto {
            id: row.id as f64,
            name: row.name,
            track_count: row.track_count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_artist_tracks(
    id: f64,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .artist_tracks(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows.into_iter().map(track_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_genres(state: State<'_, LibraryState>) -> Result<Vec<GenreDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_genres()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(name, count)| GenreDto {
            name,
            track_count: count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_genre_tracks(
    name: String,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    browse_tracks(&state, "genre", &name)
}

#[tauri::command]
#[specta::specta]
pub fn library_years(state: State<'_, LibraryState>) -> Result<Vec<YearDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_years()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(year, count)| YearDto {
            year: year as f64,
            track_count: count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_year_tracks(
    year: f64,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    browse_tracks(&state, "year", &(year as i64).to_string())
}

#[tauri::command]
#[specta::specta]
pub fn library_folders(state: State<'_, LibraryState>) -> Result<Vec<FolderDto>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .list_folders()
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(id, path, count)| FolderDto {
            id: id as f64,
            path,
            track_count: count as f64,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_folder_tracks(
    id: f64,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    browse_tracks(&state, "folder", &(id as i64).to_string())
}

fn browse_tracks(
    state: &State<'_, LibraryState>,
    filter: &str,
    value: &str,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let guard = lock_db(state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = database
        .browse_tracks(filter, value)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(rows.into_iter().map(track_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_album_cover(
    id: f64,
    state: State<'_, LibraryState>,
) -> Result<Option<String>, CommandError> {
    let guard = lock_db(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(None);
    };
    let Some(path) = database
        .album_cover_path(id as i64)
        .map_err(|err| CommandError::internal(err.to_string()))?
    else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path).map_err(|err| CommandError::internal(err.to_string()))?;
    let mime = match Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
    {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("tiff") => "image/tiff",
        _ => "image/jpeg",
    };
    Ok(Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}
