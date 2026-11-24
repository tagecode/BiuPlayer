use std::time::{SystemTime, UNIX_EPOCH};

use biu_ipc::{AppInfo, AudioDevice, PingReply, TestTick};
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const TEST_TICK: &str = "test:tick";

#[tauri::command]
#[specta::specta]
pub fn app_info(app: AppHandle) -> AppInfo {
    let package = app.package_info();
    let log_dir = app
        .path()
        .app_log_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<unavailable>".into());
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

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}
