use std::path::Path;

use biu_ipc::CommandError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LyricLineDto {
    pub time_ms: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LyricsDto {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub by: Option<String>,
    pub offset_ms: f64,
    pub lines: Vec<LyricLineDto>,
}

#[tauri::command]
#[specta::specta]
pub fn lyrics_load(path: String) -> Result<Option<LyricsDto>, CommandError> {
    let audio = Path::new(&path);
    if !audio.is_file() {
        return Err(CommandError::invalid_argument("audio file not found"));
    }
    match biu_lyrics::load_sidecar(audio) {
        Ok(lyrics) => Ok(Some(LyricsDto {
            title: lyrics.title,
            artist: lyrics.artist,
            album: lyrics.album,
            by: lyrics.by,
            offset_ms: lyrics.offset_ms as f64,
            lines: lyrics
                .lines
                .into_iter()
                .map(|line| LyricLineDto {
                    time_ms: line.time_ms as f64,
                    text: line.text,
                })
                .collect(),
        })),
        Err(biu_lyrics::LyricsError::NotFound) => Ok(None),
        Err(err) => Err(CommandError::internal(err.to_string())),
    }
}

#[tauri::command]
#[specta::specta]
pub fn desktop_lyrics_toggle(app: tauri::AppHandle) -> Result<bool, CommandError> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(window) = app.get_webview_window("desktop-lyrics") {
        let visible = window
            .is_visible()
            .map_err(|err| CommandError::internal(err.to_string()))?;
        if visible {
            window
                .hide()
                .map_err(|err| CommandError::internal(err.to_string()))?;
            return Ok(false);
        }
        window
            .show()
            .map_err(|err| CommandError::internal(err.to_string()))?;
        window
            .set_focus()
            .map_err(|err| CommandError::internal(err.to_string()))?;
        return Ok(true);
    }

    WebviewWindowBuilder::new(
        &app,
        "desktop-lyrics",
        WebviewUrl::App("index.html#/desktop-lyrics".into()),
    )
    .title("BiuPlayer 桌面歌词")
    .inner_size(880.0, 180.0)
    .min_inner_size(480.0, 120.0)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(true)
    .build()
    .map_err(|err| CommandError::internal(err.to_string()))?;
    Ok(true)
}
