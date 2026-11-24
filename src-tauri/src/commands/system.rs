use std::time::{SystemTime, UNIX_EPOCH};

use biu_ipc::{AppInfo, AppPathsInfo, AudioDevice, PingReply, TestTick};
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::AppPathsState;

pub const TEST_TICK: &str = "test:tick";

#[tauri::command]
#[specta::specta]
pub fn app_info(app: AppHandle, paths: State<'_, AppPathsState>) -> AppInfo {
    let package = app.package_info();
    let log_dir = paths.0.log_dir.display().to_string();
    AppInfo {
        name: package.name.clone(),
        version: package.version.to_string(),
        tauri_version: tauri::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        log_dir,
    }
}

#[tauri::command]
#[specta::specta]
pub fn app_ping(payload: String) -> PingReply {
    let echo = format!("pong:{}", payload.trim());
    PingReply {
        payload,
        echo,
        server_time_ms: now_ms(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn emit_test_events(app: AppHandle, count: u32) -> Result<u32, String> {
    emit_test_events_impl(&app, count)
}

pub fn emit_test_events_impl<R: Runtime>(app: &AppHandle<R>, count: u32) -> Result<u32, String> {
    let count = count.min(10_000);
    for seq in 1..=count {
        app.emit(
            TEST_TICK,
            TestTick {
                seq,
                sent_at_ms: now_ms(),
            },
        )
        .map_err(|e| e.to_string())?;
    }
    tracing::info!(count, "emitted test events");
    Ok(count)
}

#[tauri::command]
#[specta::specta]
pub fn audio_devices() -> Result<Vec<AudioDevice>, String> {
    biu_audio::list_output_devices()
        .map(|devices| {
            devices
                .into_iter()
                .map(|device| AudioDevice {
                    name: device.name,
                    is_default: device.is_default,
                    channels: device.channels,
                    sample_rates: device.sample_rates,
                    sample_formats: device.sample_formats,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn app_paths(paths: State<'_, AppPathsState>) -> AppPathsInfo {
    AppPathsInfo {
        data_dir: paths.0.data_dir.display().to_string(),
        cache_dir: paths.0.cache_dir.display().to_string(),
        log_dir: paths.0.log_dir.display().to_string(),
        settings_file: paths.0.settings_file.display().to_string(),
        database_file: paths.0.database_file.display().to_string(),
        portable: paths.0.portable,
    }
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}
