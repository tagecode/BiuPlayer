//! Cross-platform application paths and portable-mode resolution.
//!
//! The rules follow `docs/Arch.md` §12.1. Portable mode is enabled by placing a
//! `portable.flag` file next to the executable; all mutable state then lives in
//! `<exe dir>/data`.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub settings_file: PathBuf,
    pub database_file: PathBuf,
    pub backups_dir: PathBuf,
    pub portable: bool,
}

impl AppPaths {
    /// Resolve paths from the executable directory and the process environment.
    pub fn resolve(exe_dir: &Path) -> Self {
        let portable = exe_dir.join("portable.flag").is_file();
        if portable {
            return Self::from_roots(exe_dir.join("data"), exe_dir.join("data/cache"), true);
        }

        let (data, cache) = default_roots(exe_dir);
        Self::from_roots(data, cache, false)
    }

    /// Build a path set from explicit roots. Useful for tests and for callers
    /// that already resolved platform directories through another API.
    pub fn from_roots(data_dir: PathBuf, cache_dir: PathBuf, portable: bool) -> Self {
        Self {
            log_dir: data_dir.join("logs"),
            settings_file: data_dir.join("settings.json"),
            database_file: data_dir.join("library.db"),
            backups_dir: data_dir.join("backups"),
            data_dir,
            cache_dir,
            portable,
        }
    }

    /// Create every directory the application owns. Safe to call repeatedly.
    pub fn ensure(&self) -> io::Result<()> {
        for dir in [
            &self.data_dir,
            &self.cache_dir,
            &self.log_dir,
            &self.backups_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn default_roots(exe_dir: &Path) -> (PathBuf, PathBuf) {
    let data = env_path("APPDATA")
        .map(|base| base.join("BiuPlayer"))
        .unwrap_or_else(|| exe_dir.join("data"));
    let cache = env_path("LOCALAPPDATA")
        .map(|base| base.join("BiuPlayer").join("cache"))
        .unwrap_or_else(|| data.join("cache"));
    (data, cache)
}

#[cfg(target_os = "macos")]
fn default_roots(exe_dir: &Path) -> (PathBuf, PathBuf) {
    match env_path("HOME") {
        Some(home) => (
            home.join("Library/Application Support/BiuPlayer"),
            home.join("Library/Caches/BiuPlayer"),
        ),
        None => fallback_roots(exe_dir),
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn default_roots(exe_dir: &Path) -> (PathBuf, PathBuf) {
    let data = env_path("XDG_DATA_HOME")
        .map(|base| base.join("biuplayer"))
        .or_else(|| env_path("HOME").map(|home| home.join(".local/share/biuplayer")));
    let cache = env_path("XDG_CACHE_HOME")
        .map(|base| base.join("biuplayer"))
        .or_else(|| env_path("HOME").map(|home| home.join(".cache/biuplayer")));
    match (data, cache) {
        (Some(data), Some(cache)) => (data, cache),
        _ => fallback_roots(exe_dir),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
fn default_roots(exe_dir: &Path) -> (PathBuf, PathBuf) {
    fallback_roots(exe_dir)
}

fn fallback_roots(exe_dir: &Path) -> (PathBuf, PathBuf) {
    let data = exe_dir.join("data");
    let cache = data.join("cache");
    (data, cache)
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(tag: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("biu-paths-{}-{}-{}", std::process::id(), tag, id));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn portable_flag_moves_state_next_to_executable() {
        let exe_dir = temp_dir("portable");
        std::fs::write(exe_dir.join("portable.flag"), b"").unwrap();

        let paths = AppPaths::resolve(&exe_dir);

        assert!(paths.portable);
        assert_eq!(paths.data_dir, exe_dir.join("data"));
        assert_eq!(paths.cache_dir, exe_dir.join("data/cache"));
        assert_eq!(paths.settings_file, exe_dir.join("data/settings.json"));
        assert_eq!(paths.database_file, exe_dir.join("data/library.db"));
        assert_eq!(paths.log_dir, exe_dir.join("data/logs"));
    }

    #[test]
    fn ensure_creates_owned_directories() {
        let exe_dir = temp_dir("ensure");
        std::fs::write(exe_dir.join("portable.flag"), b"").unwrap();
        let paths = AppPaths::resolve(&exe_dir);

        paths.ensure().unwrap();

        assert!(paths.data_dir.is_dir());
        assert!(paths.cache_dir.is_dir());
        assert!(paths.log_dir.is_dir());
        assert!(paths.backups_dir.is_dir());
        assert!(!paths.settings_file.exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_default_uses_application_support() {
        let home = PathBuf::from(std::env::var_os("HOME").expect("HOME"));
        let exe_dir = temp_dir("mac-default");
        let paths = AppPaths::resolve(&exe_dir);

        assert!(!paths.portable);
        assert_eq!(
            paths.data_dir,
            home.join("Library/Application Support/BiuPlayer")
        );
        assert_eq!(paths.cache_dir, home.join("Library/Caches/BiuPlayer"));
    }
}
