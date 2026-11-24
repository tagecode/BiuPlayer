use std::path::PathBuf;

use biu_audio::probe_path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: probe <file>...");
        std::process::exit(2);
    }

    let mut failures = 0;
    for arg in args {
        let path = PathBuf::from(&arg);
        match probe_path(&path) {
            Ok(info) => println!(
                "OK\t{}\t{}\t{}\t{}Hz\t{}ch\t{}ms\t{} frames\t{:.1} dBFS",
                info.path,
                info.format.as_str(),
                info.codec,
                info.sample_rate,
                info.channels,
                info.duration_ms,
                info.decoded_frames,
                info.peak_dbfs
            ),
            Err(err) => {
                failures += 1;
                println!("FAIL\t{}\t{err}", path.display());
            }
        }
    }

    if failures > 0 {
        std::process::exit(1);
    }
}
