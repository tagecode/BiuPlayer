pub mod commands;
mod logging;

#[cfg(debug_assertions)]
use std::path::PathBuf;

use biu_ipc::{AppInfo, AudioDevice, PingReply, TestTick};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

#[cfg(debug_assertions)]
fn bindings_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts")
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::system::app_info,
            commands::system::app_ping,
            commands::system::emit_test_events,
            commands::system::audio_devices,
        ])
        .typ::<AppInfo>()
        .typ::<PingReply>()
        .typ::<TestTick>()
        .typ::<AudioDevice>()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), bindings_path())
        .expect("failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let (guard, log_dir) = logging::init(app.handle())
                .map_err(|err| std::io::Error::other(format!("logging init failed: {err}")))?;
            app.manage(guard);
            println!("BiuPlayer logs: {}", log_dir.display());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_typescript_bindings() {
        specta_builder()
            .export(Typescript::default(), bindings_path())
            .expect("bindings export");
    }
}
