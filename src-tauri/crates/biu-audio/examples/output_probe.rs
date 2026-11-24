use biu_audio::{list_output_devices, play_sine};

fn main() {
    match list_output_devices() {
        Ok(devices) => {
            for device in devices {
                println!(
                    "{}\tdefault={}\t{}ch\t{:?}\t{:?}",
                    device.name,
                    device.is_default,
                    device.channels,
                    device.sample_rates,
                    device.sample_formats
                );
            }
        }
        Err(err) => eprintln!("device enumeration failed: {err}"),
    }

    let rate = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(44_100);
    match play_sine(None, rate, 1.0, 440.0) {
        Ok(info) => println!(
            "PLAYED\t{}\t{}Hz\t{}ch\t{}\t{}s",
            info.device, info.sample_rate, info.channels, info.sample_format, info.seconds
        ),
        Err(err) => {
            eprintln!("playback failed: {err}");
            std::process::exit(1);
        }
    }
}
