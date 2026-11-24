use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use biu_library::Database;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "biu-library-test-{}-{}-{}",
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
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 2);
    assert_eq!(db.track_count().unwrap(), 2);

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_reads_tags_and_lists_tracks() {
    use lofty::config::WriteOptions;
    use lofty::file::{AudioFile, TaggedFileExt};
    use lofty::tag::{Accessor, Tag, TagType};

    let root = temp_root();
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/audio/m0/tone.mp3");
    let target = root.join("tagged.mp3");
    std::fs::copy(source, &target).unwrap();

    let mut tagged = lofty::read_from_path(&target).unwrap();
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title("测试标题".to_string());
    tag.set_artist("测试艺术家".to_string());
    tag.set_album("测试专辑".to_string());
    tag.set_track(3);
    tag.set_year(2024);
    tagged.insert_tag(tag);
    tagged
        .save_to_path(&target, WriteOptions::default())
        .unwrap();

    let mut db = Database::open(root.join("test.db")).unwrap();
    let folder_id = db.add_folder(&root).unwrap();
    db.scan_folder(folder_id, &root).unwrap();

    let tracks = db.list_tracks(10, 0).unwrap();
    assert_eq!(tracks.len(), 1);
    let track = &tracks[0];
    assert_eq!(track.title, "测试标题");
    assert_eq!(track.artist.as_deref(), Some("测试艺术家"));
    assert_eq!(track.album.as_deref(), Some("测试专辑"));
    assert!(track.duration_ms.unwrap_or(0) >= 1900);
    assert_eq!(track.format.as_deref(), Some("mp3"));

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_pauses_and_resumes_without_duplicates() {
    use biu_library::{ScanControl, ScanOutcome, ScanProgress};

    let root = temp_root();
    for index in 0..20 {
        std::fs::write(root.join(format!("track-{index:02}.mp3")), []).unwrap();
    }
    let mut db = Database::open(root.join("test.db")).unwrap();
    let folder_id = db.add_folder(&root).unwrap();

    let control = ScanControl::default();
    control.pause_after(5);
    let progress = ScanProgress::default();
    let first = db
        .scan_folder_controlled(folder_id, &root, &control, &progress)
        .unwrap();
    assert!(matches!(first, ScanOutcome::Paused(_)));
    assert_eq!(db.track_count().unwrap(), 5);

    let resume = ScanControl::default();
    let second = db
        .scan_folder_controlled(folder_id, &root, &resume, &progress)
        .unwrap();
    assert!(matches!(second, ScanOutcome::Completed(_)));
    assert_eq!(db.track_count().unwrap(), 20);

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn searches_by_pinyin_and_artist_album() {
    use lofty::config::WriteOptions;
    use lofty::file::{AudioFile, TaggedFileExt};
    use lofty::tag::{Accessor, Tag, TagType};

    let root = temp_root();
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/audio/m0/tone.mp3");
    let target = root.join("sunny.mp3");
    std::fs::copy(source, &target).unwrap();

    let mut tagged = lofty::read_from_path(&target).unwrap();
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title("晴天".to_string());
    tag.set_artist("周杰伦".to_string());
    tag.set_album("叶惠美".to_string());
    tagged.insert_tag(tag);
    tagged
        .save_to_path(&target, WriteOptions::default())
        .unwrap();

    let mut db = Database::open(root.join("test.db")).unwrap();
    let folder_id = db.add_folder(&root).unwrap();
    db.scan_folder(folder_id, &root).unwrap();

    for query in [
        "qingtian",
        "qt",
        "zhoujielun",
        "zjl",
        "yehuimei",
        "yhm",
        "周杰伦",
        "晴天",
    ] {
        let hits = db.search(query, 10).unwrap();
        assert!(!hits.is_empty(), "query {query} should hit");
    }

    drop(db);
    std::fs::remove_dir_all(&root).unwrap();
}
