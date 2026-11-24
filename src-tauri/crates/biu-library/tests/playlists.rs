use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use biu_library::Database;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "biu-playlist-test-{}-{}-{}",
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

#[test]
fn playlist_crud_and_ordering() {
    let root = temp_root();
    for name in ["a.mp3", "b.mp3", "c.mp3"] {
        std::fs::write(root.join(name), []).unwrap();
    }
    let mut db = Database::open_in_memory().unwrap();
    let folder = db.add_folder(&root).unwrap();
    db.scan_folder(folder, &root).unwrap();
    let tracks = db.list_tracks(10, 0).unwrap();
    assert_eq!(tracks.len(), 3);

    let id = db.create_playlist("Favorites").unwrap();
    assert!(db
        .create_playlist("  ")
        .unwrap_err()
        .to_string()
        .contains("empty"));
    assert_eq!(
        db.add_tracks_to_playlist(id, &[tracks[0].id, tracks[2].id])
            .unwrap(),
        2
    );
    assert_eq!(db.add_tracks_to_playlist(id, &[tracks[0].id]).unwrap(), 0);

    let items = db.playlist_items(id).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].position, 0);
    assert_eq!(items[1].position, 1);
    assert_eq!(items[0].track.path, tracks[0].path);

    db.move_playlist_item(id, 0, 1).unwrap();
    let items = db.playlist_items(id).unwrap();
    assert_eq!(items[0].track.path, tracks[2].path);

    db.remove_playlist_item(id, 0).unwrap();
    let items = db.playlist_items(id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].position, 0);

    db.rename_playlist(id, "Road trip").unwrap();
    let list = db.list_playlists().unwrap();
    assert_eq!(list[0].name, "Road trip");
    assert_eq!(list[0].track_count, 1);

    db.delete_playlist(id).unwrap();
    assert!(db.list_playlists().unwrap().is_empty());
    assert!(db.playlist_items(id).unwrap().is_empty());

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn playlist_copy_bulk_remove_and_reorder() {
    let root = temp_root();
    for name in ["a.mp3", "b.mp3", "c.mp3"] {
        std::fs::write(root.join(name), []).unwrap();
    }
    let mut db = Database::open_in_memory().unwrap();
    let folder = db.add_folder(&root).unwrap();
    db.scan_folder(folder, &root).unwrap();
    let tracks = db.list_tracks(10, 0).unwrap();
    let id = db.create_playlist("Source").unwrap();
    db.add_tracks_to_playlist(id, &[tracks[0].id, tracks[1].id, tracks[2].id])
        .unwrap();

    let copied = db.copy_playlist(id, "Copy").unwrap();
    assert_ne!(copied, id);
    let items = db.playlist_items(copied).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].track.path, tracks[0].path);
    assert!(db
        .copy_playlist(id, " ")
        .unwrap_err()
        .to_string()
        .contains("empty"));

    let ids: Vec<i64> = db
        .playlist_items(id)
        .unwrap()
        .into_iter()
        .map(|item| item.track.id)
        .collect();
    db.reorder_playlist_items(id, &[ids[2], ids[0], ids[1]])
        .unwrap();
    let items = db.playlist_items(id).unwrap();
    assert_eq!(items[0].track.path, tracks[2].path);
    assert!(db
        .reorder_playlist_items(id, &[ids[0]])
        .unwrap_err()
        .to_string()
        .contains("exactly"));

    assert_eq!(db.remove_playlist_items(id, &[0, 2]).unwrap(), 2);
    let items = db.playlist_items(id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].position, 0);

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}
