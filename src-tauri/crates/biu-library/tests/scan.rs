use std::path::PathBuf;

use biu_library::Database;

fn temp_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "biu-library-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn scan_inserts_and_searches() {
    let root = temp_root();
    for name in ["track-one.mp3", "track-two.flac", "notes.txt"] {
        std::fs::write(root.join(name), []).unwrap();
    }

    let mut db = Database::open(root.join("test.db")).unwrap();
    let folder_id = db.add_folder(&root).unwrap();
    let first = db.scan_folder(folder_id, &root).unwrap();
    assert_eq!(first.files_seen, 2);
    assert_eq!(db.track_count().unwrap(), 2);

    let hits = db.search("track", 10).unwrap();
    assert_eq!(hits.len(), 2);

    let second = db.scan_folder(folder_id, &root).unwrap();
    assert_eq!(second.files_seen, 2);
    assert_eq!(db.track_count().unwrap(), 2);

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}
