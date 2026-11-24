use std::path::Path;
use std::sync::Mutex;

use biu_ipc::{CommandError, LibraryScanDto, LibraryTrackDto};
use biu_library::{Database, ScanReport};
use tauri::State;

#[derive(Default)]
pub struct LibraryState(pub Mutex<Option<Database>>);

fn lock(state: &LibraryState) -> std::sync::MutexGuard<'_, Option<Database>> {
    state
        .0
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

#[tauri::command]
#[specta::specta]
pub fn library_add_folder(
    path: String,
    state: State<'_, LibraryState>,
) -> Result<LibraryScanDto, CommandError> {
    let root = Path::new(&path);
    if !root.is_dir() {
        return Err(CommandError::invalid_argument(
            "folder does not exist or is not a directory",
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|err| CommandError::invalid_argument(err.to_string()))?;

    let mut guard = lock(&state);
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

#[tauri::command]
#[specta::specta]
pub fn library_tracks(
    limit: u32,
    offset: u32,
    state: State<'_, LibraryState>,
) -> Result<Vec<LibraryTrackDto>, CommandError> {
    let guard = lock(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let tracks = database
        .list_tracks(limit.min(1000) as usize, offset as usize)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(tracks
        .into_iter()
        .map(|track| LibraryTrackDto {
            id: track.id as f64,
            title: track.title,
            artist: track.artist,
            album: track.album,
            duration_ms: track.duration_ms.map(|value| value as f64),
            format: track.format,
            path: track.path,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn library_track_count(state: State<'_, LibraryState>) -> f64 {
    let guard = lock(&state);
    guard
        .as_ref()
        .and_then(|database| database.track_count().ok())
        .unwrap_or(0) as f64
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
    let guard = lock(&state);
    let Some(database) = guard.as_ref() else {
        return Ok(Vec::new());
    };
    let tracks = database
        .search(query, limit.min(200) as usize)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(tracks
        .into_iter()
        .map(|track| LibraryTrackDto {
            id: track.id as f64,
            title: track.title,
            artist: None,
            album: None,
            duration_ms: None,
            format: Some(track.format),
            path: track.path,
        })
        .collect())
}
