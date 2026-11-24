//! Crash logging and support bundles.
//!
//! Diagnostics deliberately contain configuration, logs and system metadata,
//! never the music library database or user audio files.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use biu_core::AppPaths;
use serde_json::json;
use zip::write::SimpleFileOptions;

const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

pub fn install_panic_hook(log_dir: PathBuf) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = if let Some(text) = info.payload().downcast_ref::<&str>() {
            (*text).to_string()
        } else if let Some(text) = info.payload().downcast_ref::<String>() {
            text.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "<unknown>".into());
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>").to_string();
        let backtrace = std::backtrace::Backtrace::force_capture();
        let entry = format!(
            "timestamp={}\nthread={thread_name}\nlocation={location}\npanic={payload}\n\n{backtrace}\n",
            timestamp()
        );

        let path = log_dir.join(format!("crash-{}.log", timestamp()));
        if let Err(err) = fs::write(&path, &entry) {
            eprintln!("failed to write crash log {}: {err}", path.display());
        }
        eprintln!("{entry}");
        default_hook(info);
    }));
}

/// Create `diagnostics/biu-diagnostics-<ts>.zip` inside the data directory.
pub fn export_diagnostics(paths: &AppPaths, app_version: &str) -> Result<PathBuf, String> {
    paths.ensure().map_err(|err| err.to_string())?;
    let out_dir = paths.data_dir.join("diagnostics");
    fs::create_dir_all(&out_dir).map_err(|err| err.to_string())?;
    let out = out_dir.join(format!("biu-diagnostics-{}.zip", timestamp()));

    let file = File::create(&out).map_err(|err| err.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let settings = biu_core::load_settings(&paths.settings_file);
    let system = json!({
        "appVersion": app_version,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "portable": paths.portable,
        "dataDir": paths.data_dir,
        "cacheDir": paths.cache_dir,
        "logDir": paths.log_dir,
        "settingsSchema": settings.settings.schema_version,
        "settingsReadOnly": settings.read_only,
        "databaseExists": paths.database_file.is_file(),
    });
    write_entry(
        &mut zip,
        options,
        "system.json",
        serde_json::to_string_pretty(&system)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )?;

    if paths.settings_file.is_file() {
        copy_entry(&mut zip, options, "settings.json", &paths.settings_file)?;
    }

    if paths.log_dir.is_dir() {
        let mut logs: Vec<_> = fs::read_dir(&paths.log_dir)
            .map_err(|err| err.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(".log"))
            })
            .collect();
        logs.sort();
        for (index, log) in logs.iter().rev().take(5).enumerate() {
            let name = log
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("biu.log");
            let mut data = Vec::new();
            File::open(log)
                .and_then(|file| file.take(MAX_LOG_BYTES).read_to_end(&mut data))
                .map_err(|err| format!("read {}: {err}", log.display()))?;
            let entry = format!("logs/{index}-{name}");
            write_entry(&mut zip, options, &entry, &data)?;
        }
    }

    write_entry(
        &mut zip,
        options,
        "README.txt",
        b"BiuPlayer diagnostics bundle.\nContains settings, recent logs and system metadata.\nNo audio files or library database are included.\n",
    )?;

    zip.finish().map_err(|err| err.to_string())?;
    Ok(out)
}

fn write_entry(
    zip: &mut zip::ZipWriter<File>,
    options: SimpleFileOptions,
    name: &str,
    data: &[u8],
) -> Result<(), String> {
    zip.start_file(name, options)
        .map_err(|err| err.to_string())?;
    zip.write_all(data).map_err(|err| err.to_string())?;
    Ok(())
}

fn copy_entry(
    zip: &mut zip::ZipWriter<File>,
    options: SimpleFileOptions,
    name: &str,
    path: &Path,
) -> Result<(), String> {
    let data = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    write_entry(zip, options, name, &data)
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn diagnostics_include_settings_logs_and_metadata() {
        let root = std::env::temp_dir().join(format!("biu-diag-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let paths = AppPaths::from_roots(root.join("data"), root.join("cache"), true);
        paths.ensure().unwrap();
        fs::write(paths.settings_file.clone(), br#"{"schemaVersion":1}"#).unwrap();
        fs::write(paths.log_dir.join("biu.log.2026-01-01"), b"hello log").unwrap();

        let out = export_diagnostics(&paths, "0.1.0").unwrap();

        let mut archive = zip::ZipArchive::new(File::open(&out).unwrap()).unwrap();
        let mut names = Vec::new();
        for index in 0..archive.len() {
            names.push(archive.by_index(index).unwrap().name().to_string());
        }
        assert!(names.contains(&"system.json".to_string()));
        assert!(names.contains(&"settings.json".to_string()));
        assert!(names.iter().any(|name| name.starts_with("logs/")));
        assert!(names.contains(&"README.txt".to_string()));

        let mut system = String::new();
        archive
            .by_name("system.json")
            .unwrap()
            .read_to_string(&mut system)
            .unwrap();
        assert!(system.contains("\"appVersion\": \"0.1.0\""));
        assert!(system.contains("\"portable\": true"));
    }
}
