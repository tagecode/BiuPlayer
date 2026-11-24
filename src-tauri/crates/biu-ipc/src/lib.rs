use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub tauri_version: String,
    pub os: String,
    pub arch: String,
    pub log_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PingReply {
    pub payload: String,
    pub echo: String,
    pub server_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TestTick {
    pub seq: u32,
    pub sent_at_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppPathsInfo {
    pub data_dir: String,
    pub cache_dir: String,
    pub log_dir: String,
    pub settings_file: String,
    pub database_file: String,
    pub portable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub channels: u16,
    pub sample_rates: Vec<u32>,
    pub sample_formats: Vec<String>,
}

pub mod error_codes {
    pub const E_INTERNAL: &str = "E_INTERNAL";
    pub const E_INVALID_ARGUMENT: &str = "E_INVALID_ARGUMENT";
    pub const E_AUDIO_DEVICE: &str = "E_AUDIO_DEVICE";
    pub const E_WINAMP: &str = "E_WINAMP";
}

/// Stable error envelope shared by every Tauri command.
///
/// The frontend switches on `code`; `message` is for logs and toasts. New codes
/// are additive, existing codes must not change meaning.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(error_codes::E_INTERNAL, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(error_codes::E_INVALID_ARGUMENT, message)
    }

    pub fn audio_device(message: impl Into<String>) -> Self {
        Self::new(error_codes::E_AUDIO_DEVICE, message)
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

/// Snapshot of the playback engine, shared by commands and the
/// `player:state` event.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PlayerStateDto {
    pub state: String,
    pub path: Option<String>,
    pub position_ms: f64,
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub volume: f32,
    pub error: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub muted: bool,
    pub mode: String,
    pub queue_index: Option<f64>,
    pub queue_len: f64,
    pub finished: bool,
}

impl PlayerStateDto {
    pub fn idle() -> Self {
        Self {
            state: "idle".to_string(),
            path: None,
            position_ms: 0.0,
            duration_ms: 0.0,
            sample_rate: 0,
            channels: 0,
            volume: 1.0,
            error: None,
            title: None,
            artist: None,
            album: None,
            muted: false,
            mode: "sequential".to_string(),
            queue_index: None,
            queue_len: 0.0,
            finished: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct QueueItemDto {
    pub path: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct QueueSnapshotDto {
    pub items: Vec<QueueItemDto>,
    pub index: Option<f64>,
    pub mode: String,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LibraryScanDto {
    pub files_seen: f64,
    pub inserted: f64,
    pub updated: f64,
    pub skipped: f64,
    pub elapsed_ms: f64,
    pub total_tracks: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LibraryScanStatusDto {
    pub state: String,
    pub root: Option<String>,
    pub files_seen: f64,
    pub inserted: f64,
    pub skipped: f64,
    pub total_tracks: f64,
    pub error: Option<String>,
}

impl Default for LibraryScanStatusDto {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            root: None,
            files_seen: 0.0,
            inserted: 0.0,
            skipped: 0.0,
            total_tracks: 0.0,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LibraryTrackDto {
    pub id: f64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<f64>,
    pub format: Option<String>,
    pub path: String,
}
