use std::path::{Path, PathBuf};

use biu_audio::decode::{probe_metadata, AudioError};
use biu_core::AudioFormat;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/audio")
        .join(name)
}

#[test]
fn empty_file_is_rejected_without_probing() {
    let err = probe_metadata(fixture("bad/empty.mp3")).unwrap_err();
    assert!(matches!(err, AudioError::Empty), "got {err:?}");
}

#[test]
fn text_file_is_not_audio() {
    let err = probe_metadata(fixture("bad/text.mp3")).unwrap_err();
    assert!(
        matches!(err, AudioError::Unsupported(_) | AudioError::Corrupt(_)),
        "got {err:?}"
    );
}

#[test]
fn truncated_file_is_rejected() {
    let err = probe_metadata(fixture("bad/truncated.flac")).unwrap_err();
    assert!(
        matches!(err, AudioError::Corrupt(_) | AudioError::Unsupported(_)),
        "got {err:?}"
    );
}

#[test]
fn wrong_extension_falls_back_to_content() {
    let info = probe_metadata(fixture("bad/wrong-ext.mp3")).expect("wav content");
    assert_eq!(info.format, AudioFormat::Wav);
    assert_eq!(info.sample_rate, 44_100);
    assert_eq!(info.channels, 2);
}

#[test]
fn metadata_matches_full_decode() {
    let fast = probe_metadata(fixture("m0/tone.wav")).expect("metadata");
    let full = biu_audio::decode::probe_path(fixture("m0/tone.wav")).expect("full decode");
    assert_eq!(fast.sample_rate, full.sample_rate);
    assert_eq!(fast.channels, full.channels);
    assert_eq!(fast.duration_ms, full.duration_ms);
    assert_eq!(fast.format, full.format);
}
