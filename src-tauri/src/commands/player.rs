use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use biu_audio::{PlaybackState, Player, PlayerSnapshot};
use biu_ipc::{CommandError, PlayerStateDto, QueueItemDto, QueueSnapshotDto};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::queue::{AudioQueue, PlayMode};

pub const PLAYER_STATE: &str = "player:state";

static NEXT_SEQ: AtomicU64 = AtomicU64::new(1);

/// Lazily created playback engine. Creating it opens the default output device,
/// so the app can still start on machines without an audio device.
#[derive(Default)]
pub struct AudioPlayerState(pub Mutex<Option<Player>>);

pub struct AudioQueueState(pub Mutex<AudioQueue>);

fn lock(state: &AudioPlayerState) -> std::sync::MutexGuard<'_, Option<Player>> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn queue_lock(state: &AudioQueueState) -> std::sync::MutexGuard<'_, AudioQueue> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn snapshot_of(player: &Player) -> PlayerSnapshot {
    player.snapshot()
}

fn to_dto(snapshot: PlayerSnapshot, queue: &AudioQueue) -> PlayerStateDto {
    let current = queue.current();
    PlayerStateDto {
        state: snapshot.state.as_str().to_string(),
        path: snapshot.path,
        position_ms: snapshot.position_ms as f64,
        duration_ms: snapshot.duration_ms as f64,
        sample_rate: snapshot.sample_rate,
        channels: snapshot.channels,
        volume: snapshot.volume,
        error: snapshot.error,
        title: current.map(|item| item.title.clone()),
        artist: current.and_then(|item| item.artist.clone()),
        album: current.and_then(|item| item.album.clone()),
        muted: queue.muted(),
        mode: queue.mode().as_str().to_string(),
        queue_index: queue.index().map(|index| index as f64),
        queue_len: queue.items().len() as f64,
        finished: snapshot.finished,
        device_lost: snapshot.device_lost,
        seq: 0.0,
    }
}

pub fn snapshot(state: &AudioPlayerState, queue: &AudioQueueState) -> PlayerStateDto {
    let guard = lock(state);
    let queue = queue_lock(queue);
    let mut dto = match guard.as_ref() {
        Some(player) => to_dto(snapshot_of(player), &queue),
        None => {
            let mut dto = PlayerStateDto::idle();
            dto.mode = queue.mode().as_str().to_string();
            dto.muted = queue.muted();
            dto.queue_index = queue.index().map(|index| index as f64);
            dto.queue_len = queue.items().len() as f64;
            if let Some(item) = queue.current() {
                dto.title = Some(item.title.clone());
                dto.artist = item.artist.clone();
                dto.album = item.album.clone();
            }
            dto.seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed) as f64;
            dto
        }
    };
    dto.seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed) as f64;
    dto
}

fn control(
    state: &AudioPlayerState,
    queue: &AudioQueueState,
    action: impl FnOnce(&Player),
) -> PlayerStateDto {
    {
        let guard = lock(state);
        if let Some(player) = guard.as_ref() {
            action(player);
        }
    }
    snapshot(state, queue)
}

fn supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "oga" | "aiff" | "aif"
            )
        })
        .unwrap_or(false)
}

fn validate_item(item: &QueueItemDto) -> Result<(), CommandError> {
    let path = Path::new(&item.path);
    if !path.is_file() {
        return Err(CommandError::invalid_argument(format!(
            "file does not exist: {}",
            item.path
        )));
    }
    if !supported_extension(path) {
        return Err(CommandError::invalid_argument(format!(
            "unsupported audio extension: {}",
            item.path
        )));
    }
    Ok(())
}

fn wait_until_loaded(
    player_state: &AudioPlayerState,
    queue_state: &AudioQueueState,
) -> PlayerStateDto {
    for _ in 0..50 {
        let current = snapshot(player_state, queue_state);
        if current.state != PlaybackState::Loading.as_str() {
            return current;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    snapshot(player_state, queue_state)
}

fn play_index(
    index: usize,
    player_state: &AudioPlayerState,
    queue_state: &AudioQueueState,
) -> Result<PlayerStateDto, CommandError> {
    let (path, muted) = {
        let mut queue = queue_lock(queue_state);
        queue.set_index(index);
        let path = queue
            .current()
            .map(|item| item.path.clone())
            .ok_or_else(|| CommandError::invalid_argument("queue index out of range"))?;
        (path, queue.muted())
    };

    {
        let mut guard = lock(player_state);
        if guard.is_none() {
            *guard =
                Some(Player::new().map_err(|err| CommandError::audio_device(err.to_string()))?);
        }
        let player = guard.as_ref().expect("player just created");
        player.load(&path);
        if muted {
            player.set_volume(0.0);
        }
        player.play();
    }

    Ok(wait_until_loaded(player_state, queue_state))
}

#[tauri::command]
#[specta::specta]
pub fn player_open(
    path: String,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    let candidate = Path::new(&path);
    if !candidate.is_file() {
        return Err(CommandError::invalid_argument("file does not exist"));
    }
    if !supported_extension(candidate) {
        return Err(CommandError::invalid_argument(
            "unsupported audio extension for MVP",
        ));
    }

    {
        let mut guard = lock(&player);
        if guard.is_none() {
            *guard =
                Some(Player::new().map_err(|err| CommandError::audio_device(err.to_string()))?);
        }
        let player = guard.as_ref().expect("player just created");
        player.load(Path::new(&path));
        player.play();
    }

    Ok(wait_until_loaded(&player, &queue_state))
}

#[tauri::command]
#[specta::specta]
pub fn player_play_queue(
    items: Vec<QueueItemDto>,
    start_index: f64,
    mode: String,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    if items.is_empty() {
        return Err(CommandError::invalid_argument("queue is empty"));
    }
    for item in &items {
        if !supported_extension(Path::new(&item.path)) {
            return Err(CommandError::invalid_argument(format!(
                "unsupported audio extension: {}",
                item.path
            )));
        }
    }
    let start = start_index.max(0.0) as usize;
    if start >= items.len() {
        return Err(CommandError::invalid_argument("start index out of range"));
    }
    let mode = PlayMode::parse(&mode)
        .ok_or_else(|| CommandError::invalid_argument("unknown play mode"))?;
    queue_lock(&queue_state).replace(items, start, mode);
    play_index(start, &player, &queue_state)
}

#[tauri::command]
#[specta::specta]
pub fn player_play(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    control(&player, &queue_state, |player| player.play())
}

#[tauri::command]
#[specta::specta]
pub fn player_pause(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    control(&player, &queue_state, |player| player.pause())
}

#[tauri::command]
#[specta::specta]
pub fn player_stop(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    control(&player, &queue_state, |player| player.stop())
}

#[tauri::command]
#[specta::specta]
pub fn player_seek(
    position_ms: f64,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    control(&player, &queue_state, |player| {
        player.seek_ms(position_ms.max(0.0) as u64)
    })
}

#[tauri::command]
#[specta::specta]
pub fn player_set_volume(
    volume: f32,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    queue_lock(&queue_state).set_muted(false);
    control(&player, &queue_state, |player| player.set_volume(volume))
}

#[tauri::command]
#[specta::specta]
pub fn player_set_muted(
    muted: bool,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    queue_lock(&queue_state).set_muted(muted);
    let volume = if muted { 0.0 } else { 1.0 };
    control(&player, &queue_state, |player| player.set_volume(volume))
}

#[tauri::command]
#[specta::specta]
pub fn player_next(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    let next = queue_lock(&queue_state).next_index();
    match next {
        Some(index) => play_index(index, &player, &queue_state),
        None => {
            if let Some(player) = lock(&player).as_ref() {
                player.acknowledge_finished();
                player.stop();
            }
            Ok(snapshot(&player, &queue_state))
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn player_prev(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    let previous = queue_lock(&queue_state).prev_index();
    match previous {
        Some(index) => play_index(index, &player, &queue_state),
        None => Ok(snapshot(&player, &queue_state)),
    }
}

#[tauri::command]
#[specta::specta]
pub fn player_jump(
    index: f64,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    let index = index.max(0.0) as usize;
    if index >= queue_lock(&queue_state).items().len() {
        return Err(CommandError::invalid_argument("queue index out of range"));
    }
    play_index(index, &player, &queue_state)
}

#[tauri::command]
#[specta::specta]
pub fn player_set_mode(
    mode: String,
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> Result<PlayerStateDto, CommandError> {
    let mode = PlayMode::parse(&mode)
        .ok_or_else(|| CommandError::invalid_argument("unknown play mode"))?;
    queue_lock(&queue_state).set_mode(mode);
    Ok(snapshot(&player, &queue_state))
}

#[tauri::command]
#[specta::specta]
pub fn player_queue(queue_state: State<'_, AudioQueueState>) -> QueueSnapshotDto {
    queue_lock(&queue_state).snapshot()
}

#[tauri::command]
#[specta::specta]
pub fn player_queue_add(
    item: QueueItemDto,
    queue_state: State<'_, AudioQueueState>,
) -> Result<QueueSnapshotDto, CommandError> {
    validate_item(&item)?;
    let mut queue = queue_lock(&queue_state);
    queue.add(item);
    Ok(queue.snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn player_queue_remove(
    index: f64,
    queue_state: State<'_, AudioQueueState>,
) -> Result<QueueSnapshotDto, CommandError> {
    let mut queue = queue_lock(&queue_state);
    queue
        .remove(index.max(0.0) as usize)
        .ok_or_else(|| CommandError::invalid_argument("queue index out of range"))?;
    Ok(queue.snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn player_queue_clear(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    queue_lock(&queue_state).clear();
    if let Some(player) = lock(&player).as_ref() {
        player.stop();
    }
    snapshot(&player, &queue_state)
}

#[tauri::command]
#[specta::specta]
pub fn player_snapshot(
    player: State<'_, AudioPlayerState>,
    queue_state: State<'_, AudioQueueState>,
) -> PlayerStateDto {
    snapshot(&player, &queue_state)
}

fn recover_output(
    player_state: &AudioPlayerState,
    queue_state: &AudioQueueState,
) -> Result<(), String> {
    let (path, position, muted) = {
        let guard = lock(player_state);
        let player = guard.as_ref().ok_or("no active player")?;
        let snapshot = player.snapshot();
        let path = snapshot.path.ok_or("no active track")?;
        let muted = queue_lock(queue_state).muted();
        (path, snapshot.position_ms, muted)
    };

    *lock(player_state) = None;
    let player = Player::new().map_err(|err| err.to_string())?;
    player.load(&path);
    if muted {
        player.set_volume(0.0);
    }
    player.play();
    if position > 0 {
        player.seek_ms(position);
    }
    *lock(player_state) = Some(player);
    Ok(())
}

/// Emits `player:state` at ~4 Hz. Handles EOF auto-advance, bad-file skipping
/// (up to three consecutive failures) and output-device loss recovery.
pub fn spawn_player_events(app: AppHandle) {
    std::thread::spawn(move || {
        let mut failures = 0u32;
        let mut last_recovery = std::time::Instant::now();
        loop {
            std::thread::sleep(Duration::from_millis(250));
            let player_state = app.state::<AudioPlayerState>();
            let queue_state = app.state::<AudioQueueState>();
            let current = snapshot(&player_state, &queue_state);

            if current.device_lost {
                if last_recovery.elapsed() >= Duration::from_secs(2) {
                    last_recovery = std::time::Instant::now();
                    match recover_output(&player_state, &queue_state) {
                        Ok(()) => {
                            failures = 0;
                            tracing::warn!("output device lost; rebuilt default stream");
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "cannot rebuild output device");
                        }
                    }
                }
            } else if current.finished {
                failures = 0;
                let next = queue_lock(&queue_state).next_index();
                match next {
                    Some(index) => {
                        if let Err(err) = play_index(index, &player_state, &queue_state) {
                            tracing::warn!(error = %err, "auto advance failed");
                        }
                    }
                    None => {
                        if let Some(player) = lock(&player_state).as_ref() {
                            player.acknowledge_finished();
                            player.stop();
                        }
                    }
                }
            } else if current.state == PlaybackState::Error.as_str() {
                failures += 1;
                tracing::warn!(
                    path = %current.path.as_deref().unwrap_or("-"),
                    error = %current.error.as_deref().unwrap_or("unknown"),
                    failures,
                    "playback failed"
                );
                if failures >= 3 {
                    failures = 0;
                    if let Some(player) = lock(&player_state).as_ref() {
                        player.stop();
                    }
                    tracing::error!("stopped after three consecutive playback failures");
                } else if let Some(index) = queue_lock(&queue_state).next_index() {
                    if let Err(err) = play_index(index, &player_state, &queue_state) {
                        tracing::warn!(error = %err, "skip after playback failure failed");
                    }
                }
            } else if matches!(current.state.as_str(), "playing" | "paused" | "loading") {
                failures = 0;
            }

            let dto = snapshot(&player_state, &queue_state);
            if app.emit(PLAYER_STATE, dto).is_err() {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_snapshot_without_player() {
        let state = AudioPlayerState::default();
        let path = std::env::temp_dir().join("biu-player-idle-queue.json");
        let queue = AudioQueueState(Mutex::new(AudioQueue::new(path)));
        let dto = snapshot(&state, &queue);
        assert_eq!(dto.state, "idle");
        assert_eq!(dto.position_ms, 0.0);
        assert_eq!(dto.mode, "sequential");
    }
}
