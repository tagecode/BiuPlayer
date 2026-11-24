use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tauri::Listener;

#[test]
fn emits_1000_events_without_loss() {
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let handle = app.handle().clone();
    let received = Arc::new(AtomicU32::new(0));
    let counter = received.clone();
    let _id = handle.listen(biuplayer_lib::commands::system::TEST_TICK, move |_| {
        counter.fetch_add(1, Ordering::Relaxed);
    });

    let sent =
        biuplayer_lib::commands::system::emit_test_events_impl(&handle, 1000).expect("emit events");
    assert_eq!(sent, 1000);
    assert_eq!(received.load(Ordering::Relaxed), 1000);
}

#[test]
fn ping_echoes_payload() {
    let reply = biuplayer_lib::commands::system::app_ping("biu".into());
    assert_eq!(reply.echo, "pong:biu");
}

#[test]
fn rejects_oversized_event_burst_with_stable_code() {
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");

    let error = biuplayer_lib::commands::system::emit_test_events_impl(app.handle(), 10_001)
        .expect_err("must reject");
    assert_eq!(error.code, biu_ipc::error_codes::E_INVALID_ARGUMENT);

    let json = serde_json::to_value(&error).unwrap();
    assert_eq!(json["code"], "E_INVALID_ARGUMENT");
    assert!(json["message"].as_str().unwrap().contains("10000"));
}

#[test]
fn player_snapshot_is_idle_without_device() {
    use tauri::Manager;

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    app.manage(biuplayer_lib::commands::player::AudioPlayerState::default());
    app.manage(biuplayer_lib::commands::player::AudioQueueState(
        std::sync::Mutex::new(biuplayer_lib::queue::AudioQueue::new(
            std::env::temp_dir().join("biu-ipc-idle-queue.json"),
        )),
    ));

    let snapshot = biuplayer_lib::commands::player::player_snapshot(app.state(), app.state());
    assert_eq!(snapshot.state, "idle");
    assert_eq!(snapshot.position_ms, 0.0);
}

#[test]
fn player_open_rejects_missing_file() {
    use tauri::Manager;

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    app.manage(biuplayer_lib::commands::player::AudioPlayerState::default());
    app.manage(biuplayer_lib::commands::player::AudioQueueState(
        std::sync::Mutex::new(biuplayer_lib::queue::AudioQueue::new(
            std::env::temp_dir().join("biu-ipc-open-queue.json"),
        )),
    ));

    let error = biuplayer_lib::commands::player::player_open(
        "/definitely/not/a/track.mp3".to_string(),
        app.state(),
        app.state(),
    )
    .expect_err("missing file must fail");
    assert_eq!(error.code, biu_ipc::error_codes::E_INVALID_ARGUMENT);
}

#[test]
fn queue_commands_persist_items_without_opening_audio() {
    use std::path::PathBuf;
    use tauri::Manager;

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let session = std::env::temp_dir().join(format!("biu-queue-ipc-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&session);
    app.manage(biuplayer_lib::commands::player::AudioQueueState(
        std::sync::Mutex::new(biuplayer_lib::queue::AudioQueue::new(session)),
    ));

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/audio/m0/tone.flac");
    let item = biu_ipc::QueueItemDto {
        path: fixture.display().to_string(),
        title: "tone".to_string(),
        artist: Some("biu".to_string()),
        album: None,
        duration_ms: Some(2000.0),
    };
    let snapshot = biuplayer_lib::commands::player::player_queue_add(item, app.state())
        .expect("add queue item");
    assert_eq!(snapshot.items.len(), 1);
    assert_eq!(snapshot.index, Some(0.0));

    let snapshot = biuplayer_lib::commands::player::player_queue_remove(0.0, app.state())
        .expect("remove queue item");
    assert!(snapshot.items.is_empty());
    assert_eq!(snapshot.index, None);
}

#[test]
fn library_add_folder_scans_fixtures() {
    use std::path::PathBuf;
    use tauri::Manager;

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let database = biu_library::Database::open_in_memory().unwrap();
    app.manage(biuplayer_lib::commands::library::LibraryState::new(
        Some(database),
        PathBuf::from(":memory:"),
    ));

    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/audio/m0");
    let report = biuplayer_lib::commands::library::library_add_folder(
        fixtures.display().to_string(),
        app.state(),
    )
    .expect("scan fixtures");
    assert_eq!(report.files_seen, 7.0);
    assert_eq!(report.total_tracks, 7.0);

    let tracks =
        biuplayer_lib::commands::library::library_tracks(100, 0, app.state()).expect("list");
    assert_eq!(tracks.len(), 7);
    assert!(tracks
        .iter()
        .any(|track| track.format.as_deref() == Some("flac")));
}

#[test]
fn library_scan_start_reports_progress_until_complete() {
    use std::path::PathBuf;
    use tauri::Manager;

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let db_path = std::env::temp_dir().join(format!(
        "biu-scan-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.manage(biuplayer_lib::commands::library::LibraryState::new(
        None,
        db_path.clone(),
    ));
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/audio/m0");

    let started = biuplayer_lib::commands::library::library_scan_start(
        fixtures.display().to_string(),
        app.state(),
    )
    .expect("start scan");
    assert_eq!(started.state, "scanning");

    let mut final_state = String::new();
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let status = biuplayer_lib::commands::library::library_scan_status(app.state());
        final_state = status.state;
        if final_state == "completed" || final_state == "error" {
            assert_eq!(status.files_seen, 7.0, "{:?}", status.error);
            break;
        }
    }
    assert_eq!(final_state, "completed");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
}
