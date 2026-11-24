use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use biu_audio::{PlaybackState, Player, PlayerSnapshot};
use biu_ipc::{CommandError, PlayerStateDto};
use tauri::{AppHandle, Emitter, Manager, State};

pub const PLAYER_STATE: &str = "player:state";

/// Lazily created playback engine. Creating it opens the default output device,
/// so the app can still start on machines without an audio device.
#[derive(Default)]
pub struct AudioPlayerState(pub Mutex<Option<Player>>);

fn lock(state: &AudioPlayerState) -> std::sync::MutexGuard<'_, Option<Player>> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn to_dto(snapshot: PlayerSnapshot) -> PlayerStateDto {
    PlayerStateDto {
        state: snapshot.state.as_str().to_string(),
        path: snapshot.path,
        position_ms: snapshot.position_ms as f64,
        duration_ms: snapshot.duration_ms as f64,
        sample_rate: snapshot.sample_rate,
        channels: snapshot.channels,
        volume: snapshot.volume,
        error: snapshot.error,
    }
}

pub fn snapshot(state: &AudioPlayerState) -> PlayerStateDto {
    let guard = lock(state);
    match guard.as_ref() {
        Some(player) => to_dto(player.snapshot()),
        None => PlayerStateDto::idle(),
    }
}

fn control(state: &AudioPlayerState, action: impl FnOnce(&Player)) -> PlayerStateDto {
    let guard = lock(state);
    if let Some(player) = guard.as_ref() {
        action(player);
    }
    drop(guard);
    snapshot(state)
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

#[tauri::command]
#[specta::specta]
pub fn player_open(
    path: String,
    state: State<'_, AudioPlayerState>,
) -> Result<PlayerStateDto, CommandError> {
    let path = Path::new(&path);
    if !path.is_file() {
        return Err(CommandError::invalid_argument("file does not exist"));
    }
    if !supported_extension(path) {
        return Err(CommandError::invalid_argument(
            "unsupported audio extension for MVP",
        ));
    }

    {
        let mut guard = lock(&state);
        if guard.is_none() {
            let player =
                Player::new().map_err(|err| CommandError::audio_device(err.to_string()))?;
            *guard = Some(player);
        }
        let player = guard.as_ref().expect("player just created");
        player.load(path);
        player.play();
    }

    // Decoding and device setup are asynchronous; wait briefly so the caller
    // gets Loading/Playing instead of an immediate Idle snapshot.
    for _ in 0..50 {
        let current = snapshot(&state);
        if current.state != PlaybackState::Loading.as_str() {
            return Ok(current);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(snapshot(&state))
}

#[tauri::command]
#[specta::specta]
pub fn player_play(state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    control(&state, |player| player.play())
}

#[tauri::command]
#[specta::specta]
pub fn player_pause(state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    control(&state, |player| player.pause())
}

#[tauri::command]
#[specta::specta]
pub fn player_stop(state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    control(&state, |player| player.stop())
}

#[tauri::command]
#[specta::specta]
pub fn player_seek(position_ms: f64, state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    control(&state, |player| player.seek_ms(position_ms.max(0.0) as u64))
}

#[tauri::command]
#[specta::specta]
pub fn player_set_volume(volume: f32, state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    control(&state, |player| player.set_volume(volume))
}

#[tauri::command]
#[specta::specta]
pub fn player_snapshot(state: State<'_, AudioPlayerState>) -> PlayerStateDto {
    snapshot(&state)
}

/// Emits `player:state` at ~4 Hz so the UI never has to poll.
pub fn spawn_player_events(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(250));
        let dto = {
            let state = app.state::<AudioPlayerState>();
            snapshot(&state)
        };
        if app.emit(PLAYER_STATE, dto).is_err() {
            break;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_snapshot_without_player() {
        let state = AudioPlayerState::default();
        let dto = snapshot(&state);
        assert_eq!(dto.state, "idle");
        assert_eq!(dto.position_ms, 0.0);
    }
}
