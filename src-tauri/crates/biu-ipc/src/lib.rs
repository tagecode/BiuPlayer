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
