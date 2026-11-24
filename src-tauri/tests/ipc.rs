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

    let snapshot = biuplayer_lib::commands::player::player_snapshot(app.state());
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

    let error = biuplayer_lib::commands::player::player_open(
        "/definitely/not/a/track.mp3".to_string(),
        app.state(),
    )
    .expect_err("missing file must fail");
    assert_eq!(error.code, biu_ipc::error_codes::E_INVALID_ARGUMENT);
}
