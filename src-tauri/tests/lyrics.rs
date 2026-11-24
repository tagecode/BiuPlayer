use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use biuplayer_lib::commands::lyrics::lyrics_load;

fn temp_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "biu-lyrics-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn loads_sidecar_lrc_with_offset() {
    let dir = temp_dir();
    let audio = dir.join("晴天.mp3");
    fs::write(&audio, b"not audio").unwrap();
    fs::write(
        audio.with_extension("lrc"),
        "[ti:晴天]\n[offset:-200]\n[00:01.00]第一行\n[00:02.00]第二行\n",
    )
    .unwrap();

    let lyrics = lyrics_load(audio.to_string_lossy().into_owned())
        .unwrap()
        .expect("lyrics");
    assert_eq!(lyrics.title.as_deref(), Some("晴天"));
    assert_eq!(lyrics.offset_ms, -200.0);
    assert_eq!(lyrics.lines.len(), 2);
    assert_eq!(lyrics.lines[0].time_ms, 800.0);
    assert_eq!(lyrics.lines[1].text, "第二行");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn returns_none_without_sidecar() {
    let dir = temp_dir();
    let audio = dir.join("none.mp3");
    fs::write(&audio, b"").unwrap();
    assert!(lyrics_load(audio.to_string_lossy().into_owned())
        .unwrap()
        .is_none());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn falls_back_to_embedded_lyrics() {
    use lofty::file::{AudioFile, TaggedFileExt};
    use lofty::tag::ItemKey;

    let dir = temp_dir();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/audio/m0/tone.mp3");
    let audio = dir.join("embedded.mp3");
    fs::copy(&source, &audio).unwrap();
    let mut tagged = lofty::read_from_path(&audio).unwrap();
    tagged
        .primary_tag_mut()
        .unwrap()
        .insert_text(ItemKey::Lyrics, "[00:01.00]内嵌第一行".to_string());
    tagged
        .save_to_path(&audio, lofty::config::WriteOptions::default())
        .unwrap();

    let lyrics = lyrics_load(audio.to_string_lossy().into_owned())
        .unwrap()
        .expect("embedded lyrics");
    assert_eq!(lyrics.source, "embedded");
    assert_eq!(lyrics.lines[0].text, "内嵌第一行");
    fs::remove_dir_all(&dir).unwrap();
}
