//! Real-output smoke test (requires an audio device; run manually).
//!
//! ```text
//! cargo test -p biu-audio --test output_smoke -- --ignored --nocapture
//! ```

use biu_audio::output::{list_output_devices, play_sine};

#[test]
#[ignore = "requires a working audio output device"]
fn default_device_enumerates_and_plays() {
    let devices = list_output_devices().expect("enumerate output devices");
    assert!(!devices.is_empty(), "no output devices reported");
    assert!(
        devices.iter().any(|device| device.is_default),
        "no device reported as default"
    );

    let info = play_sine(None, 48_000, 0.3, 440.0).expect("play sine on default device");
    assert!(
        info.sample_rate >= 8_000,
        "unexpected rate {}",
        info.sample_rate
    );
    assert!(info.channels >= 1);
    assert!(
        (info.seconds - 0.3).abs() < 0.15,
        "expected ~0.3s of output, got {}",
        info.seconds
    );
}
