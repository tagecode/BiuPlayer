//! TAG-01 coverage: write real tags with lofty, then verify the scanner reads
//! them back for each container/format the MVP promises.

use std::fs;
use std::path::{Path, PathBuf};

use biu_library::Database;
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};

const TITLE: &str = "晴天";
const ARTIST: &str = "周杰伦";
const ALBUM: &str = "叶惠美";
const ALBUM_ARTIST: &str = "杰威尔音乐";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/audio/m0")
        .join(name)
}

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "biu-tags-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_tag(src: &Path, dst: &Path, tag_type: TagType, with_album_artist: bool) {
    fs::copy(src, dst).unwrap_or_else(|err| panic!("copy {}: {err}", src.display()));
    // Drop whatever tags the fixture shipped with, otherwise lofty keeps the
    // original container as `primary_tag` and the new tag is never read.
    let existing = lofty::read_from_path(dst).unwrap();
    for tag in existing.tags() {
        Tag::new(tag.tag_type()).remove_from_path(dst).unwrap();
    }
    let mut tag = Tag::new(tag_type);
    // ID3v1 is Latin-1 only, so it gets ASCII text; the other containers keep
    // the Chinese values used across the MVP fixtures.
    let ascii = tag_type == TagType::Id3v1;
    tag.set_title(if ascii {
        "Qingtian".to_string()
    } else {
        TITLE.to_string()
    });
    tag.set_artist(if ascii {
        "Jay".to_string()
    } else {
        ARTIST.to_string()
    });
    tag.set_album(if ascii {
        "Yehuimei".to_string()
    } else {
        ALBUM.to_string()
    });
    tag.set_track(3);
    tag.set_genre("Mandopop".to_string());
    tag.set_year(2003);
    if with_album_artist {
        tag.insert_text(ItemKey::AlbumArtist, ALBUM_ARTIST.to_string());
    }
    tag.save_to_path(dst, WriteOptions::default())
        .unwrap_or_else(|err| panic!("write {tag_type:?} tag to {}: {err}", dst.display()));
}

/// Scan a single tagged file and return the album-artist names the library
/// reports. Also asserts the basic text fields survived the round trip.
fn scan_one(name: &str, tag_type: TagType, with_album_artist: bool) -> Vec<String> {
    let (exp_title, exp_artist, exp_album) = if tag_type == TagType::Id3v1 {
        ("Qingtian", "Jay", "Yehuimei")
    } else {
        (TITLE, ARTIST, ALBUM)
    };
    let root = temp_root(name);
    let song = root.join(name);
    write_tag(&fixture(name), &song, tag_type, with_album_artist);

    let mut db = Database::open(root.join("library.db")).unwrap();
    let folder = db.add_folder(&root).unwrap();
    let report = db.scan_folder(folder, &root).unwrap();
    assert_eq!(report.files_seen, 1, "{name}: file should be scanned");

    let hits = db.search(exp_artist, 5).unwrap();
    assert_eq!(hits.len(), 1, "{name}: search by artist should hit");
    assert_eq!(hits[0].title, exp_title, "{name}: title");
    assert_eq!(
        hits[0].artist.as_deref(),
        Some(exp_artist),
        "{name}: artist"
    );
    assert_eq!(hits[0].album.as_deref(), Some(exp_album), "{name}: album");

    db.list_album_artists()
        .unwrap()
        .into_iter()
        .map(|row| row.name)
        .collect()
}

#[test]
fn reads_id3v2_apev2_vorbis_mp4_and_wav_tags() {
    // MP3/ID3v2
    let names = scan_one("tone.mp3", TagType::Id3v2, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "the native album artist should be present"
    );
    assert!(!names.contains(&ARTIST.to_string()));

    // MP3/APEv2
    let names = scan_one("tone.mp3", TagType::Ape, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "APE album artist should be present"
    );

    // FLAC/Vorbis comments
    let names = scan_one("tone.flac", TagType::VorbisComments, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "Vorbis album artist should be present"
    );

    // OGG/Vorbis comments
    let names = scan_one("tone.ogg", TagType::VorbisComments, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "OGG album artist should be present"
    );

    // M4A/MP4 atoms
    let names = scan_one("tone-aac.m4a", TagType::Mp4Ilst, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "MP4 album artist should be present"
    );

    // WAV/ID3v2 chunk
    let names = scan_one("tone.wav", TagType::Id3v2, true);
    assert!(
        names.contains(&ALBUM_ARTIST.to_string()),
        "WAV ID3 album artist should be present"
    );
}

#[test]
fn reads_id3v1_and_falls_back_to_artist() {
    let names = scan_one("tone.mp3", TagType::Id3v1, false);
    assert_eq!(
        names,
        vec!["Jay".to_string()],
        "ID3v1 has no album artist field"
    );
}
