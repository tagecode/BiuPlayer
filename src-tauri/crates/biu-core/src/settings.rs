//! `settings.json` model, validation, atomic writes and corrupt-file recovery.

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub general: GeneralSettings,
    pub playback: PlaybackSettings,
    pub audio: AudioSettings,
    pub dsp: DspSettings,
    pub lyrics: LyricsSettings,
    pub library: LibrarySettings,
    pub ui: UiSettings,
    /// Unknown fields are preserved so a newer minor config is not destroyed.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralSettings::default(),
            playback: PlaybackSettings::default(),
            audio: AudioSettings::default(),
            dsp: DspSettings::default(),
            lyrics: LyricsSettings::default(),
            library: LibrarySettings::default(),
            ui: UiSettings::default(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GeneralSettings {
    pub language: String,
    pub theme: String,
    pub skin: String,
    pub start_minimized: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".into(),
            theme: "system".into(),
            skin: "builtin.dark".into(),
            start_minimized: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlaybackSettings {
    pub mode: String,
    pub crossfade_ms: u32,
    pub gapless: bool,
    pub resume_on_start: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            mode: "sequential".into(),
            crossfade_ms: 0,
            gapless: true,
            resume_on_start: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioSettings {
    pub device_id: String,
    pub buffer_ms: u32,
    pub bit_perfect: bool,
    pub replay_gain: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            device_id: "default".into(),
            buffer_ms: 15,
            bit_perfect: false,
            replay_gain: "album".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DspSettings {
    pub eq_enabled: bool,
    pub eq_preset: String,
    pub preamp_db: f32,
    pub mono: bool,
}

impl Default for DspSettings {
    fn default() -> Self {
        Self {
            eq_enabled: false,
            eq_preset: "flat".into(),
            preamp_db: 0.0,
            mono: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LyricsSettings {
    pub desktop: bool,
    pub font_size: u32,
    pub desktop_color: String,
    pub offset_ms: i32,
    pub source: String,
}

impl Default for LyricsSettings {
    fn default() -> Self {
        Self {
            desktop: false,
            font_size: 32,
            desktop_color: "#f5f7fa".into(),
            offset_ms: 0,
            source: "local".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibrarySettings {
    pub watch: bool,
    pub exclude_globs: Vec<String>,
    pub clean_missing: bool,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            watch: true,
            exclude_globs: vec!["**/@eaDir/**".into()],
            clean_missing: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiSettings {
    pub sidebar_width: u32,
    pub list_density: String,
    pub reduce_motion: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_ui_opacity")]
    pub opacity: f64,
}

fn default_ui_opacity() -> f64 {
    1.0
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            sidebar_width: 220,
            list_density: "comfortable".into(),
            reduce_motion: false,
            always_on_top: false,
            opacity: 1.0,
        }
    }
}

#[derive(Debug)]
pub enum SettingsWarning {
    Corrupt { backup: PathBuf },
    NewerVersion { found: u32, supported: u32 },
}

impl fmt::Display for SettingsWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupt { backup } => write!(
                f,
                "settings.json is invalid; backed up to {} and defaults were loaded",
                backup.display()
            ),
            Self::NewerVersion { found, supported } => write!(
                f,
                "settings schema {found} is newer than supported {supported}; settings are read-only"
            ),
        }
    }
}

#[derive(Debug)]
pub struct LoadedSettings {
    pub settings: Settings,
    pub warning: Option<SettingsWarning>,
    /// Newer schema versions must not be overwritten by an older application.
    pub read_only: bool,
}

/// Load `path`, recovering from missing/corrupt files.
///
/// A corrupt file is renamed to `settings.corrupt.<unix-seconds>.json` before
/// defaults are returned. A newer `schemaVersion` is treated as read-only.
pub fn load_settings(path: &Path) -> LoadedSettings {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return LoadedSettings {
                settings: Settings::default(),
                warning: None,
                read_only: false,
            }
        }
        Err(err) => {
            let backup = quarantine_corrupt(path);
            tracing::warn!(%err, backup = %backup.display(), "failed to read settings");
            return LoadedSettings {
                settings: Settings::default(),
                warning: Some(SettingsWarning::Corrupt { backup }),
                read_only: false,
            };
        }
    };

    match serde_json::from_str::<Settings>(&raw) {
        Ok(settings) => {
            if settings.schema_version > CURRENT_SCHEMA_VERSION {
                return LoadedSettings {
                    warning: Some(SettingsWarning::NewerVersion {
                        found: settings.schema_version,
                        supported: CURRENT_SCHEMA_VERSION,
                    }),
                    settings,
                    read_only: true,
                };
            }
            LoadedSettings {
                settings,
                warning: None,
                read_only: false,
            }
        }
        Err(err) => {
            let backup = quarantine_corrupt(path);
            tracing::warn!(%err, backup = %backup.display(), "invalid settings JSON");
            LoadedSettings {
                settings: Settings::default(),
                warning: Some(SettingsWarning::Corrupt { backup }),
                read_only: false,
            }
        }
    }
}

/// Atomically replace `path` with `settings`, keeping the previous file as
/// `settings.json.bak`.
pub fn save_settings(path: &Path, settings: &Settings) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "settings path has no parent",
        )
    })?;
    fs::create_dir_all(parent)?;

    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    let payload = serde_json::to_vec_pretty(settings)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(&payload)?;
        file.sync_all()?;
    }

    if path.exists() {
        let bak = path.with_extension("json.bak");
        // Backup failures are logged but do not block the write: the new file
        // is still valid and the old data is usually recoverable from the DB.
        if let Err(err) = fs::copy(path, &bak) {
            tracing::warn!(%err, bak = %bak.display(), "failed to back up settings");
        }
    }

    #[cfg(windows)]
    if path.exists() {
        // std::fs::rename cannot replace an existing file on Windows.
        fs::remove_file(path)?;
    }

    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(err);
    }
    Ok(())
}

fn quarantine_corrupt(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut candidate = path.with_extension(format!("corrupt.{stamp}.json"));
    let mut counter = 1;
    while candidate.exists() {
        candidate = path.with_extension(format!("corrupt.{stamp}.{counter}.json"));
        counter += 1;
    }
    let _ = fs::rename(path, &candidate);
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(tag: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "biu-settings-{}-{}-{}",
            std::process::id(),
            tag,
            id
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_loads_defaults_without_warning() {
        let dir = temp_dir("missing");
        let loaded = load_settings(&dir.join("settings.json"));

        assert_eq!(loaded.settings.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(loaded.settings.general.language, "zh-CN");
        assert!(loaded.warning.is_none());
        assert!(!loaded.read_only);
    }

    #[test]
    fn save_and_load_roundtrip_preserves_unknown_fields() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("settings.json");
        let mut settings = Settings::default();
        settings.ui.sidebar_width = 280;
        settings
            .extra
            .insert("futureFlag".into(), Value::Bool(true));

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert_eq!(loaded.settings.ui.sidebar_width, 280);
        assert_eq!(
            loaded.settings.extra.get("futureFlag"),
            Some(&Value::Bool(true))
        );
        assert!(loaded.warning.is_none());
    }

    #[test]
    fn save_keeps_previous_file_as_backup() {
        let dir = temp_dir("backup");
        let path = dir.join("settings.json");
        let mut first = Settings::default();
        first.ui.sidebar_width = 200;
        save_settings(&path, &first).unwrap();

        let mut second = Settings::default();
        second.ui.sidebar_width = 300;
        save_settings(&path, &second).unwrap();

        let bak = path.with_extension("json.bak");
        assert!(bak.is_file());
        let old = load_settings(&bak);
        assert_eq!(old.settings.ui.sidebar_width, 200);
    }

    #[test]
    fn corrupt_file_is_quarantined_and_defaults_returned() {
        let dir = temp_dir("corrupt");
        let path = dir.join("settings.json");
        fs::write(&path, b"{ not json").unwrap();

        let loaded = load_settings(&path);

        assert!(!path.exists());
        assert!(matches!(
            loaded.warning,
            Some(SettingsWarning::Corrupt { .. })
        ));
        assert_eq!(loaded.settings.schema_version, CURRENT_SCHEMA_VERSION);
        let corrupt_files = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".corrupt."))
            .count();
        assert_eq!(corrupt_files, 1);
    }

    #[test]
    fn newer_schema_is_read_only() {
        let dir = temp_dir("newer");
        let path = dir.join("settings.json");
        fs::write(
            &path,
            br#"{"schemaVersion":99,"general":{"language":"en"}}"#,
        )
        .unwrap();

        let loaded = load_settings(&path);

        assert!(loaded.read_only);
        assert_eq!(loaded.settings.general.language, "en");
        assert!(matches!(
            loaded.warning,
            Some(SettingsWarning::NewerVersion { found: 99, .. })
        ));
    }
}
