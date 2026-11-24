//! LRC parsing with encoding detection and offset handling.

use std::fmt;
use std::path::Path;

use lofty::file::TaggedFileExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lyrics {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub by: Option<String>,
    pub offset_ms: i64,
    pub lines: Vec<LyricLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricLine {
    pub time_ms: i64,
    pub text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LyricsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("no lyrics found")]
    NotFound,
}

impl fmt::Display for Lyrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} lines, offset {}ms", self.lines.len(), self.offset_ms)
    }
}

pub fn find_sidecar(audio: &Path) -> Option<std::path::PathBuf> {
    let base = audio.with_extension("lrc");
    if base.is_file() {
        return Some(base);
    }
    let upper = audio.with_extension("LRC");
    if upper.is_file() {
        return Some(upper);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LyricsSource {
    Sidecar,
    Embedded,
}

/// Loads sidecar `.lrc` first, then falls back to the embedded lyrics tag.
pub fn load_for_audio(audio: &Path) -> Result<(Lyrics, LyricsSource), LyricsError> {
    if find_sidecar(audio).is_some() {
        return Ok((load_sidecar(audio)?, LyricsSource::Sidecar));
    }
    let tagged = lofty::read_from_path(audio).map_err(|_| LyricsError::NotFound)?;
    let tag = tagged
        .primary_tag()
        .or_else(|| tagged.first_tag())
        .ok_or(LyricsError::NotFound)?;
    let text = tag
        .get_string(&lofty::tag::ItemKey::Lyrics)
        .ok_or(LyricsError::NotFound)?;
    if text.trim().is_empty() {
        return Err(LyricsError::NotFound);
    }
    Ok((parse(text), LyricsSource::Embedded))
}

pub fn load_sidecar(audio: &Path) -> Result<Lyrics, LyricsError> {
    let path = find_sidecar(audio).ok_or(LyricsError::NotFound)?;
    let bytes = std::fs::read(path)?;
    Ok(parse_bytes(&bytes))
}

pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return text.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (text, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return text.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let (text, _, _) = encoding_rs::GBK.decode(bytes);
    text.into_owned()
}

pub fn parse_bytes(bytes: &[u8]) -> Lyrics {
    parse(&decode(bytes))
}

pub fn parse(input: &str) -> Lyrics {
    let mut lyrics = Lyrics {
        title: None,
        artist: None,
        album: None,
        by: None,
        offset_ms: 0,
        lines: Vec::new(),
    };

    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(meta) = parse_metadata(line) {
            match meta.0 {
                "ti" => lyrics.title = Some(meta.1),
                "ar" => lyrics.artist = Some(meta.1),
                "al" => lyrics.album = Some(meta.1),
                "by" => lyrics.by = Some(meta.1),
                "offset" => lyrics.offset_ms = meta.1.trim().parse().unwrap_or(0),
                _ => {}
            }
            continue;
        }

        let mut times = Vec::new();
        let mut rest = line;
        while let Some((time, remaining)) = take_time_tag(rest) {
            times.push(time);
            rest = remaining;
        }
        if times.is_empty() {
            continue;
        }
        let text = strip_enhanced_tags(rest).trim().to_string();
        for time in times {
            lyrics.lines.push(LyricLine {
                time_ms: time,
                text: text.clone(),
            });
        }
    }

    lyrics.lines.sort_by_key(|line| line.time_ms);
    if lyrics.offset_ms != 0 {
        for line in &mut lyrics.lines {
            line.time_ms = (line.time_ms + lyrics.offset_ms).max(0);
        }
    }
    lyrics
}

fn parse_metadata(line: &str) -> Option<(&str, String)> {
    let end = line.find(']')?;
    if !line.starts_with('[') {
        return None;
    }
    let inside = &line[1..end];
    let (key, value) = inside.split_once(':')?;
    match key {
        "ti" | "ar" | "al" | "by" | "offset" => Some((key, value.to_string())),
        _ => None,
    }
}

fn take_time_tag(input: &str) -> Option<(i64, &str)> {
    let rest = input.strip_prefix('[')?;
    let end = rest.find(']')?;
    let inside = &rest[..end];
    let time = parse_timestamp(inside)?;
    Some((time, &rest[end + 1..]))
}

fn parse_timestamp(value: &str) -> Option<i64> {
    let (minutes, seconds) = value.split_once(':')?;
    let minutes: i64 = minutes.trim().parse().ok()?;
    let seconds: f64 = seconds.trim().parse().ok()?;
    if !(0.0..60.0).contains(&seconds) {
        return None;
    }
    Some(minutes * 60_000 + (seconds * 1000.0).round() as i64)
}

fn strip_enhanced_tags(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else {
            out.push_str(&rest[start..]);
            return out;
        };
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_and_enhanced_lrc() {
        let text = "[ti:晴天]\n[ar:周杰伦]\n[offset:+500]\n[00:12.34]故事的小黄花\n[00:16.00][00:20.50]<00:16.00>从<00:16.50>出生那年就飘着\n";
        let lyrics = parse(text);
        assert_eq!(lyrics.title.as_deref(), Some("晴天"));
        assert_eq!(lyrics.artist.as_deref(), Some("周杰伦"));
        assert_eq!(lyrics.offset_ms, 500);
        assert_eq!(lyrics.lines.len(), 3);
        assert_eq!(lyrics.lines[0].time_ms, 12_840);
        assert_eq!(lyrics.lines[1].text, "从出生那年就飘着");
        assert_eq!(lyrics.lines[2].time_ms, 21_000);
    }

    #[test]
    fn decodes_gbk() {
        let (bytes, _, _) = encoding_rs::GBK.encode("[00:01.00]晴天\n[00:02.00]周杰伦");
        let lyrics = parse_bytes(&bytes);
        assert_eq!(lyrics.lines[0].text, "晴天");
        assert_eq!(lyrics.lines[1].text, "周杰伦");
    }

    #[test]
    fn ignores_invalid_lines_and_sorts() {
        let lyrics = parse("not a lyric\n[00:05.00]B\n[00:01.00]A\n");
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].text, "A");
    }
}
