use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use biu_library::Database;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "biu-mvp-flow-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        unique
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/audio/m0")
}

#[test]
fn decodes_every_mvp_format() {
    let dir = fixtures();
    for name in [
        "tone.mp3",
        "tone.flac",
        "tone.wav",
        "tone-aac.m4a",
        "tone-alac.m4a",
        "tone.ogg",
        "tone.aiff",
    ] {
        let info = biu_audio::probe_path(dir.join(name)).unwrap_or_else(|error| {
            panic!("{name} should decode: {error}");
        });
        assert_eq!(info.sample_rate, 44_100, "{name}");
        assert_eq!(info.channels, 2, "{name}");
        assert!(info.duration_ms > 1_000, "{name}: {:?}", info.duration_ms);
    }
}

#[test]
fn scan_search_playlist_and_manual_order_flow() {
    let root = temp_root();
    let db_path = root.join("library.db");
    let music = fixtures();

    let mut db = Database::open(&db_path).unwrap();
    let folder = db.add_folder(&music).unwrap();
    let report = db.scan_folder(folder, &music).unwrap();
    assert_eq!(report.files_seen, 7);
    assert_eq!(report.inserted, 7);
    assert_eq!(db.track_count().unwrap(), 7);

    let hits = db.search("tone", 20).unwrap();
    assert_eq!(hits.len(), 7, "fallback title should be searchable");

    let tracks = db.list_tracks_sorted(100, 0, "title", false).unwrap();
    assert_eq!(tracks.len(), 7);
    let ids: Vec<i64> = tracks[0..3].iter().map(|track| track.id).collect();

    let playlist = db.create_playlist("验收").unwrap();
    assert_eq!(db.add_tracks_to_playlist(playlist, &ids).unwrap(), 3);
    let reversed: Vec<i64> = ids.iter().rev().copied().collect();
    db.reorder_playlist_items(playlist, &reversed).unwrap();
    let items = db.playlist_items(playlist).unwrap();
    let ordered: Vec<i64> = items.iter().map(|item| item.track.id).collect();
    assert_eq!(ordered, reversed);

    let copy = db.copy_playlist(playlist, "验收副本").unwrap();
    assert_eq!(db.playlist_items(copy).unwrap().len(), 3);
    assert_eq!(db.remove_playlist_items(copy, &[0]).unwrap(), 1);
    assert_eq!(db.playlist_items(copy).unwrap().len(), 2);

    let all = db.list_tracks_sorted(100, 0, "manual", false).unwrap();
    let last_id = all.last().unwrap().id;
    let first = all.first().unwrap().id;
    db.move_track(last_id, None, Some(first)).unwrap();
    let manual = db.list_tracks_sorted(100, 0, "manual", false).unwrap();
    assert_eq!(manual.first().unwrap().id, last_id);

    drop(db);
    let db = Database::open(&db_path).unwrap();
    let persisted = db.list_tracks_sorted(100, 0, "manual", false).unwrap();
    assert_eq!(persisted.first().unwrap().id, last_id);
    assert_eq!(db.playlist_items(playlist).unwrap().len(), 3);

    assert!(db.remove_folder(folder).unwrap());
    assert_eq!(db.track_count().unwrap(), 0);
}
