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
