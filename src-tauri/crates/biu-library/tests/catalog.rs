use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use biu_library::Database;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, Tag, TagType};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "biu-catalog-test-{}-{}-{}",
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

fn tagged_copy(root: &Path, name: &str, title: &str, artist: &str, album: &str) {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/audio/m0/tone.mp3");
    let target = root.join(name);
    std::fs::copy(source, &target).unwrap();
    let mut tagged = lofty::read_from_path(&target).unwrap();
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(title.to_string());
    tag.set_artist(artist.to_string());
    tag.set_album(album.to_string());
    tag.set_track(1);
    tag.set_year(2024);
    tagged.insert_tag(tag);
    tagged
        .save_to_path(&target, WriteOptions::default())
        .unwrap();
}

#[test]
fn albums_and_artists_aggregate_tracks() {
    let root = temp_root();
    tagged_copy(&root, "one.mp3", "第一首", "艺术家甲", "专辑一");
    tagged_copy(&root, "two.mp3", "第二首", "艺术家甲", "专辑一");
    tagged_copy(&root, "three.mp3", "第三首", "艺术家乙", "专辑二");

    let mut db = Database::open_in_memory().unwrap();
    let folder = db.add_folder(&root).unwrap();
    db.scan_folder(folder, &root).unwrap();

    let albums = db.list_albums().unwrap();
    assert_eq!(albums.len(), 2);
    let first = albums.iter().find(|entry| entry.title == "专辑一").unwrap();
    assert_eq!(first.track_count, 2);
    assert_eq!(first.year, Some(2024));
    assert_eq!(db.album_tracks(first.id).unwrap().len(), 2);

    let artists = db.list_artists().unwrap();
    assert_eq!(artists.len(), 2);
    let artist = artists
        .iter()
        .find(|entry| entry.name == "艺术家甲")
        .unwrap();
    assert_eq!(artist.track_count, 2);
    assert_eq!(db.artist_tracks(artist.id).unwrap().len(), 2);

    std::fs::remove_dir_all(&root).unwrap();
}
