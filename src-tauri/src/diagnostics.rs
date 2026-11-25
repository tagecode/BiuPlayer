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

/// Replaces user-specific absolute paths and account names so a diagnostics
/// bundle can be shared without leaking the home directory or user name.
struct Redactor {
    pairs: Vec<(String, String)>,
}

impl Redactor {
    fn new(paths: Option<&AppPaths>) -> Self {
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Some(paths) = paths {
            pairs.push((
                paths.data_dir.to_string_lossy().into_owned(),
                "<data-dir>".to_string(),
            ));
            pairs.push((
                paths.cache_dir.to_string_lossy().into_owned(),
                "<cache-dir>".to_string(),
            ));
            pairs.push((
                paths.log_dir.to_string_lossy().into_owned(),
                "<log-dir>".to_string(),
            ));
        }
        for key in ["USERPROFILE", "HOME"] {
            if let Ok(value) = std::env::var(key) {
                if value.len() > 1 {
                    pairs.push((value, "<home>".to_string()));
                }
            }
        }
        for key in ["USERNAME", "USER"] {
            if let Ok(value) = std::env::var(key) {
                if value.chars().count() >= 3 {
                    pairs.push((value, "<user>".to_string()));
                }
            }
        }
        let mut expanded = Vec::with_capacity(pairs.len() * 3);
        for (from, to) in pairs {
            expanded.push((from.clone(), to.clone()));
            let slashed = from.replace('\\', "/");
            if slashed != from {
                expanded.push((slashed, to.clone()));
            }
            let json_escaped = from.replace('\\', "\\\\");
            if json_escaped != from {
                expanded.push((json_escaped, to));
            }
        }
        expanded.sort_by_key(|pair| std::cmp::Reverse(pair.0.len()));
        Self { pairs: expanded }
    }

    fn text(&self, input: &str) -> String {
        let mut out = input.to_string();
        for (from, to) in &self.pairs {
            if from.is_empty() {
                continue;
            }
            out = out.replace(from, to);
        }
        out
    }

    fn bytes(&self, input: &[u8]) -> Vec<u8> {
        self.text(&String::from_utf8_lossy(input)).into_bytes()
    }
}

pub fn install_panic_hook(log_dir: PathBuf) {
    let default_hook = std::panic::take_hook();
    let redactor = Redactor::new(None);
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
        let entry = redactor.text(&format!(
            "timestamp={}\nthread={thread_name}\nlocation={location}\npanic={payload}\n\n{backtrace}\n",
            timestamp()
        ));

        let path = log_dir.join(format!("crash-{}.log", timestamp_millis()));
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
    let redactor = Redactor::new(Some(paths));

    let settings = biu_core::load_settings(&paths.settings_file);
    let system = json!({
        "appVersion": app_version,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "portable": paths.portable,
        "dataDir": redactor.text(&paths.data_dir.to_string_lossy()),
        "cacheDir": redactor.text(&paths.cache_dir.to_string_lossy()),
        "logDir": redactor.text(&paths.log_dir.to_string_lossy()),
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
        let raw = fs::read(&paths.settings_file).map_err(|err| err.to_string())?;
        let redacted = redactor.bytes(&raw);
        write_entry(&mut zip, options, "settings.json", &redacted)?;
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
            write_entry(&mut zip, options, &entry, &redactor.bytes(&data))?;
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

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
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
        let mut settings = biu_core::Settings::default();
        settings.library.exclude_globs = vec![paths.data_dir.join("private").display().to_string()];
        biu_core::save_settings(&paths.settings_file, &settings).unwrap();
        fs::write(
            paths.log_dir.join("biu.log.2026-01-01"),
            format!("scan failed: {}", paths.data_dir.join("music").display()),
        )
        .unwrap();

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
        assert!(system.contains("<data-dir>"));
        assert!(!system.contains(&paths.data_dir.display().to_string()));

        let mut bundled_settings = String::new();
        archive
            .by_name("settings.json")
            .unwrap()
            .read_to_string(&mut bundled_settings)
            .unwrap();
        assert!(bundled_settings.contains("<data-dir>"));
        assert!(!bundled_settings.contains(&paths.data_dir.display().to_string()));

        let log_name = names
            .iter()
            .find(|name| name.starts_with("logs/"))
            .unwrap()
            .clone();
        let mut log = String::new();
        archive
            .by_name(&log_name)
            .unwrap()
            .read_to_string(&mut log)
            .unwrap();
        assert!(log.contains("<data-dir>"));
        assert!(!log.contains(&paths.data_dir.display().to_string()));
    }

    #[test]
    fn panic_hook_writes_crash_log_that_latest_crash_finds() {
        let log_dir = std::env::temp_dir().join(format!("biu-crash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&log_dir);
        fs::create_dir_all(&log_dir).unwrap();

        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_default();
        let previous = std::panic::take_hook();
        install_panic_hook(log_dir.clone());
        let result = std::panic::catch_unwind(|| panic!("boom in {home}"));
        std::panic::set_hook(previous);
        assert!(result.is_err());

        let report = latest_crash(&log_dir).expect("crash report should exist");
        assert!(report.modified_ms > 0.0);

        // Other tests may panic while this global hook is installed, so find the
        // entry that belongs to this test instead of assuming it is the newest.
        let mut contents = None;
        for entry in fs::read_dir(&log_dir).unwrap() {
            let path = entry.unwrap().path();
            let text = fs::read_to_string(&path).unwrap_or_default();
            if text.contains("panic=boom") {
                contents = Some(text);
                break;
            }
        }
        let contents = contents.expect("crash log for this test should exist");
        assert!(contents.contains("thread="));
        if home.chars().count() >= 3 {
            assert!(!contents.contains(&home), "home path leaked: {contents}");
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct CrashReport {
    pub file: String,
    pub modified_ms: f64,
}

/// 返回最近一次崩溃日志；没有则返回 None。前端据此提示“上次异常退出”。
pub fn latest_crash(log_dir: &Path) -> Option<CrashReport> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(log_dir).ok()?.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !name.starts_with("crash-") || !name.ends_with(".log") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if newest
            .as_ref()
            .is_none_or(|(current, _)| modified > *current)
        {
            newest = Some((modified, path));
        }
    }
    let (modified, path) = newest?;
    let modified_ms = modified
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as f64)
        .unwrap_or(0.0);
    Some(CrashReport {
        file: path.display().to_string(),
        modified_ms,
    })
}
