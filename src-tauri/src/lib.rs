pub mod commands;
mod diagnostics;
mod logging;
pub mod queue;

use std::path::Path;
#[cfg(debug_assertions)]
use std::path::PathBuf;

use biu_core::AppPaths;
use biu_ipc::{
    AppInfo, AppPathsInfo, AudioDevice, LibraryScanDto, LibraryScanStatusDto, LibraryTrackDto,
    PingReply, PlayerStateDto, TestTick,
};
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
            commands::system::app_paths,
            commands::system::export_diagnostics,
            commands::player::player_open,
            commands::player::player_play_queue,
            commands::player::player_play,
            commands::player::player_pause,
            commands::player::player_stop,
            commands::player::player_seek,
            commands::player::player_set_volume,
            commands::player::player_set_muted,
            commands::player::player_next,
            commands::player::player_prev,
            commands::player::player_jump,
            commands::player::player_set_mode,
            commands::player::player_queue,
            commands::player::player_queue_add,
            commands::player::player_queue_remove,
            commands::player::player_queue_clear,
            commands::player::player_snapshot,
            commands::library::library_add_folder,
            commands::library::library_scan_start,
            commands::library::library_scan_status,
            commands::library::library_scan_pause,
            commands::library::library_scan_resume,
            commands::library::library_scan_cancel,
            commands::library::library_tracks,
            commands::library::library_track_count,
            commands::library::library_search,
            commands::library::library_albums,
            commands::library::library_album_tracks,
            commands::library::library_artists,
            commands::library::library_artist_tracks,
            commands::library::playlist_list,
            commands::library::playlist_create,
            commands::library::playlist_rename,
            commands::library::playlist_delete,
            commands::library::playlist_items,
            commands::library::playlist_add_tracks,
            commands::library::playlist_remove_item,
            commands::library::playlist_move_item,
        ])
        .typ::<AppInfo>()
        .typ::<PingReply>()
        .typ::<TestTick>()
        .typ::<AudioDevice>()
        .typ::<AppPathsInfo>()
        .typ::<PlayerStateDto>()
        .typ::<biu_ipc::QueueItemDto>()
        .typ::<biu_ipc::QueueSnapshotDto>()
        .typ::<LibraryScanDto>()
        .typ::<LibraryScanStatusDto>()
        .typ::<LibraryTrackDto>()
        .typ::<biu_ipc::PlaylistDto>()
        .typ::<biu_ipc::PlaylistItemDto>()
        .typ::<biu_ipc::AlbumDto>()
        .typ::<biu_ipc::ArtistDto>()
}

#[derive(Clone)]
pub struct AppPathsState(pub AppPaths);

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
            let exe = std::env::current_exe().map_err(|err| {
                std::io::Error::other(format!("cannot resolve executable: {err}"))
            })?;
            let exe_dir = exe.parent().unwrap_or_else(|| Path::new("."));
            let paths = AppPaths::resolve(exe_dir);
            paths.ensure().map_err(|err| {
                std::io::Error::other(format!("cannot create app directories: {err}"))
            })?;
            diagnostics::install_panic_hook(paths.log_dir.clone());
            let (guard, log_dir) = logging::init(&paths.log_dir)
                .map_err(|err| std::io::Error::other(format!("logging init failed: {err}")))?;
            let loaded = biu_core::load_settings(&paths.settings_file);
            if let Some(warning) = &loaded.warning {
                tracing::warn!(%warning, "settings recovered");
            }
            if !loaded.read_only && !paths.settings_file.exists() {
                biu_core::save_settings(&paths.settings_file, &loaded.settings).map_err(|err| {
                    std::io::Error::other(format!("cannot write default settings: {err}"))
                })?;
                tracing::info!(settings = %paths.settings_file.display(), "default settings written");
            }
            tracing::info!(
                portable = paths.portable,
                data_dir = %paths.data_dir.display(),
                log_dir = %log_dir.display(),
                "application paths resolved"
            );
            app.manage(guard);
            let database_file = paths.database_file.clone();
            let session_file = paths.data_dir.join("player-session.json");
            app.manage(AppPathsState(paths));
            app.manage(commands::player::AudioPlayerState::default());
            app.manage(commands::player::AudioQueueState(std::sync::Mutex::new(
                crate::queue::AudioQueue::new(session_file),
            )));
            let library = match biu_library::Database::open(&database_file) {
                Ok(database) => Some(database),
                Err(err) => {
                    tracing::error!(error = %err, "failed to open library database");
                    None
                }
            };
            app.manage(commands::library::LibraryState::new(
                library,
                database_file.clone(),
            ));
            commands::player::spawn_player_events(app.handle().clone());
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
