use std::sync::Mutex;

use biu_core::Settings;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use biu_ipc::CommandError;

pub struct SettingsState {
    pub path: std::path::PathBuf,
    pub settings: Mutex<Settings>,
}

impl SettingsState {
    pub fn new(path: std::path::PathBuf, settings: Settings) -> Self {
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub schema_version: f64,
    pub language: String,
    pub theme: String,
    pub start_minimized: bool,
    pub mode: String,
    pub crossfade_ms: f64,
    pub gapless: bool,
    pub resume_on_start: bool,
    pub list_density: String,
    pub reduce_motion: bool,
    pub lyrics_offset_ms: f64,
    pub desktop_lyrics_font_size: f64,
    pub desktop_lyrics_color: String,
    pub audio_device: String,
    pub bit_perfect: bool,
    pub sidebar_width: f64,
    pub always_on_top: bool,
    pub opacity: f64,
}

impl From<&Settings> for SettingsDto {
    fn from(settings: &Settings) -> Self {
        Self {
            schema_version: settings.schema_version as f64,
            language: settings.general.language.clone(),
            theme: settings.general.theme.clone(),
            start_minimized: settings.general.start_minimized,
            mode: settings.playback.mode.clone(),
            crossfade_ms: settings.playback.crossfade_ms as f64,
            gapless: settings.playback.gapless,
            resume_on_start: settings.playback.resume_on_start,
            list_density: settings.ui.list_density.clone(),
            reduce_motion: settings.ui.reduce_motion,
            lyrics_offset_ms: settings.lyrics.offset_ms as f64,
            desktop_lyrics_font_size: settings.lyrics.font_size as f64,
            desktop_lyrics_color: settings.lyrics.desktop_color.clone(),
            audio_device: settings.audio.device_id.clone(),
            bit_perfect: settings.audio.bit_perfect,
            sidebar_width: settings.ui.sidebar_width as f64,
            always_on_top: settings.ui.always_on_top,
            opacity: settings.ui.opacity,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingsPatch {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub start_minimized: Option<bool>,
    pub mode: Option<String>,
    pub crossfade_ms: Option<f64>,
    pub gapless: Option<bool>,
    pub resume_on_start: Option<bool>,
    pub list_density: Option<String>,
    pub reduce_motion: Option<bool>,
    pub lyrics_offset_ms: Option<f64>,
    pub desktop_lyrics_font_size: Option<f64>,
    pub desktop_lyrics_color: Option<String>,
    pub audio_device: Option<String>,
    pub bit_perfect: Option<bool>,
    pub sidebar_width: Option<f64>,
    pub always_on_top: Option<bool>,
    pub opacity: Option<f64>,
}

#[tauri::command]
#[specta::specta]
pub fn settings_get(state: State<'_, SettingsState>) -> SettingsDto {
    SettingsDto::from(&*state.settings.lock().unwrap())
}

#[tauri::command]
#[specta::specta]
pub fn settings_update(
    app: tauri::AppHandle,
    patch: SettingsPatch,
    state: State<'_, SettingsState>,
) -> Result<SettingsDto, CommandError> {
    let mut guard = state.settings.lock().unwrap();
    let settings = &mut *guard;

    if let Some(language) = patch.language {
        if !matches!(language.as_str(), "zh-CN" | "zh-TW" | "en") {
            return Err(CommandError::invalid_argument("unsupported language"));
        }
        settings.general.language = language;
    }
    if let Some(theme) = patch.theme {
        if !matches!(theme.as_str(), "system" | "light" | "dark") {
            return Err(CommandError::invalid_argument("unsupported theme"));
        }
        settings.general.theme = theme;
    }
    if let Some(value) = patch.start_minimized {
        settings.general.start_minimized = value;
    }
    if let Some(mode) = patch.mode {
        if !matches!(
            mode.as_str(),
            "sequential" | "list_loop" | "single_loop" | "shuffle"
        ) {
            return Err(CommandError::invalid_argument("unsupported play mode"));
        }
        settings.playback.mode = mode;
    }
    if let Some(value) = patch.crossfade_ms {
        if !(0.0..=10_000.0).contains(&value) {
            return Err(CommandError::invalid_argument("crossfade out of range"));
        }
        settings.playback.crossfade_ms = value.round() as u32;
    }
    if let Some(value) = patch.gapless {
        settings.playback.gapless = value;
    }
    if let Some(value) = patch.resume_on_start {
        settings.playback.resume_on_start = value;
    }
    if let Some(value) = patch.list_density {
        if !matches!(value.as_str(), "comfortable" | "compact") {
            return Err(CommandError::invalid_argument("unsupported density"));
        }
        settings.ui.list_density = value;
    }
    if let Some(value) = patch.reduce_motion {
        settings.ui.reduce_motion = value;
    }
    if let Some(value) = patch.lyrics_offset_ms {
        if !(-10_000.0..=10_000.0).contains(&value) {
            return Err(CommandError::invalid_argument("lyrics offset out of range"));
        }
        settings.lyrics.offset_ms = value.round() as i32;
    }
    if let Some(value) = patch.desktop_lyrics_font_size {
        if !(16.0..=96.0).contains(&value) {
            return Err(CommandError::invalid_argument(
                "lyrics font size out of range",
            ));
        }
        settings.lyrics.font_size = value.round() as u32;
    }
    if let Some(value) = patch.audio_device {
        settings.audio.device_id = value;
    }
    if let Some(value) = patch.bit_perfect {
        settings.audio.bit_perfect = value;
    }
    if let Some(value) = patch.sidebar_width {
        if !(160.0..=400.0).contains(&value) {
            return Err(CommandError::invalid_argument("sidebar width out of range"));
        }
        settings.ui.sidebar_width = value.round() as u32;
    }
    if let Some(value) = patch.always_on_top {
        settings.ui.always_on_top = value;
    }
    if let Some(value) = patch.opacity {
        if !(0.7..=1.0).contains(&value) {
            return Err(CommandError::invalid_argument("opacity out of range"));
        }
        settings.ui.opacity = value;
    }
    if let Some(value) = patch.desktop_lyrics_color {
        let valid = value.len() == 7
            && value.starts_with('#')
            && value[1..].chars().all(|c| c.is_ascii_hexdigit());
        if !valid {
            return Err(CommandError::invalid_argument(
                "lyrics color must be #RRGGBB",
            ));
        }
        settings.lyrics.desktop_color = value;
    }

    biu_core::save_settings(&state.path, settings)
        .map_err(|err| CommandError::internal(err.to_string()))?;
    let dto = SettingsDto::from(&*settings);
    let _ = app.emit("settings:changed", ());
    Ok(dto)
}
