use std::path::Path;

use biu_audio::probe_path;

#[test]
fn decodes_all_m0_formats() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/audio/m0");
    let files = [
        "tone.mp3",
        "tone.flac",
        "tone.wav",
        "tone-aac.m4a",
        "tone-alac.m4a",
        "tone.ogg",
        "tone.aiff",
    ];

    for name in files {
        let info = probe_path(dir.join(name)).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(info.sample_rate, 44_100, "{name}");
        assert_eq!(info.channels, 2, "{name}");
        assert!(
            (1_900..=2_100).contains(&info.duration_ms),
            "{name}: duration {}",
            info.duration_ms
        );
        assert!(info.decoded_frames > 0, "{name}");
        assert!(info.peak_dbfs < 0.0, "{name}");
    }
}
