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
