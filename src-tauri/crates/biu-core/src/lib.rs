//! Pure BiuPlayer domain types and rules.
//!
//! This crate intentionally has no dependency on Tauri, SQLite, or audio APIs.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("path escapes the allowed root: {0}")]
    PathEscape(PathBuf),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

/// Canonicalize `candidate` and prove it is inside `root`.
///
/// Existing paths are canonicalized (symlinks resolved). A non-existing path is
/// absolutized and lexically normalized, then checked against the canonical root.
pub fn ensure_within(root: &Path, candidate: &Path) -> Result<PathBuf, CoreError> {
    let root = root.canonicalize()?;
    let candidate = resolve_path(candidate)?;

    if !candidate.starts_with(&root) {
        return Err(CoreError::PathEscape(candidate));
    }
    Ok(candidate)
}

fn resolve_path(path: &Path) -> Result<PathBuf, CoreError> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }

    let mut base = absolute(path)?;
    let mut suffix = Vec::new();
    while !base.exists() {
        match base.file_name() {
            Some(name) => {
                suffix.push(name.to_os_string());
                base.pop();
            }
            None => break,
        }
    }
    let mut resolved = base.canonicalize()?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn absolute(path: &Path) -> Result<PathBuf, CoreError> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(lexical_normalize(&path))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Flac,
    Wav,
    Aac,
    Alac,
    Ogg,
    Aiff,
    Unknown,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "flac" => Self::Flac,
            "wav" | "wave" => Self::Wav,
            "m4a" | "mp4" | "aac" => Self::Aac,
            "ogg" | "oga" => Self::Ogg,
            "aif" | "aiff" | "aifc" => Self::Aiff,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Aac => "aac",
            Self::Alac => "alac",
            Self::Ogg => "ogg",
            Self::Aiff => "aiff",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInfo {
    pub path: String,
    pub format: AudioFormat,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_ms: u64,
    pub decoded_frames: u64,
    pub peak_dbfs: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecodeStats {
    pub frames: u64,
    pub peak: f32,
}

impl DecodeStats {
    pub fn peak_dbfs(&self) -> f32 {
        if self.peak <= 0.0 {
            -120.0
        } else {
            20.0 * self.peak.log10()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_escape() {
        let root = std::env::temp_dir();
        let outside = std::env::temp_dir().join("../outside.txt");
        assert!(matches!(
            ensure_within(&root, &outside),
            Err(CoreError::PathEscape(_))
        ));
    }

    #[test]
    fn accepts_inside() {
        let root = std::env::temp_dir();
        let inside = root.join("biu-core-test-file");
        assert!(ensure_within(&root, &inside).is_ok());
    }
}
