use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use biu_ipc::{QueueItemDto, QueueSnapshotDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    #[default]
    Sequential,
    ListLoop,
    SingleLoop,
    Shuffle,
}

impl PlayMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "sequential" => Some(Self::Sequential),
            "list_loop" => Some(Self::ListLoop),
            "single_loop" => Some(Self::SingleLoop),
            "shuffle" => Some(Self::Shuffle),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::ListLoop => "list_loop",
            Self::SingleLoop => "single_loop",
            Self::Shuffle => "shuffle",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Session {
    items: Vec<QueueItemDto>,
    index: Option<usize>,
    mode: PlayMode,
    muted: bool,
}

pub struct AudioQueue {
    session_file: PathBuf,
    items: Vec<QueueItemDto>,
    index: Option<usize>,
    mode: PlayMode,
    muted: bool,
    shuffle_order: Vec<usize>,
    shuffle_cursor: usize,
}

impl AudioQueue {
    pub fn new(session_file: PathBuf) -> Self {
        let session = load_session(&session_file).unwrap_or_default();
        let item_count = session.items.len();
        let mut queue = Self {
            session_file,
            items: session.items,
            index: session.index.filter(|index| *index < item_count),
            mode: session.mode,
            muted: session.muted,
            shuffle_order: Vec::new(),
            shuffle_cursor: 0,
        };
        queue.rebuild_shuffle();
        queue
    }

    pub fn items(&self) -> &[QueueItemDto] {
        &self.items
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn mode(&self) -> PlayMode {
        self.mode
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn current(&self) -> Option<&QueueItemDto> {
        self.index.and_then(|index| self.items.get(index))
    }

    pub fn replace(&mut self, items: Vec<QueueItemDto>, start: usize, mode: PlayMode) {
        self.index = (!items.is_empty()).then(|| start.min(items.len() - 1));
        self.items = items;
        self.mode = mode;
        self.rebuild_shuffle();
        self.save();
    }

    pub fn add(&mut self, item: QueueItemDto) {
        self.items.push(item);
        if self.index.is_none() {
            self.index = Some(0);
        }
        self.rebuild_shuffle();
        self.save();
    }

    pub fn insert_next(&mut self, item: QueueItemDto) -> usize {
        let at = match self.index {
            Some(index) => (index + 1).min(self.items.len()),
            None => 0,
        };
        self.items.insert(at, item);
        self.save();
        at
    }

    pub fn remove(&mut self, index: usize) -> Option<QueueItemDto> {
        if index >= self.items.len() {
            return None;
        }
        let removed = self.items.remove(index);
        self.index = match self.index {
            Some(current) if current > index => Some(current - 1),
            Some(current) if current == index => {
                if self.items.is_empty() {
                    None
                } else {
                    Some(current.min(self.items.len() - 1))
                }
            }
            other => other,
        };
        self.rebuild_shuffle();
        self.save();
        Some(removed)
    }

    /// Move an item inside the queue, keeping the current track selected.
    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() || from == to {
            return false;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        if let Some(current) = self.index {
            self.index = Some(if current == from {
                to
            } else if from < current && to >= current {
                current - 1
            } else if from > current && to <= current {
                current + 1
            } else {
                current
            });
        }
        self.rebuild_shuffle();
        self.save();
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.index = None;
        self.shuffle_order.clear();
        self.shuffle_cursor = 0;
        self.save();
    }

    pub fn jump(&mut self, index: usize) -> Option<usize> {
        if index >= self.items.len() {
            return None;
        }
        self.index = Some(index);
        self.rebuild_shuffle();
        self.save();
        Some(index)
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        self.mode = mode;
        self.rebuild_shuffle();
        self.save();
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.save();
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.index = Some(index);
            self.save();
        }
    }

    /// Returns the next index without changing the current one.
    pub fn next_index(&mut self) -> Option<usize> {
        let current = self.index?;
        if self.items.is_empty() {
            return None;
        }
        match self.mode {
            PlayMode::Sequential => (current + 1 < self.items.len()).then_some(current + 1),
            PlayMode::ListLoop => Some((current + 1) % self.items.len()),
            PlayMode::SingleLoop => Some(current),
            PlayMode::Shuffle => self.next_shuffle(current),
        }
    }

    /// Manual previous wraps, except single-loop keeps the current track.
    pub fn prev_index(&mut self) -> Option<usize> {
        let current = self.index?;
        if self.items.is_empty() {
            return None;
        }
        match self.mode {
            PlayMode::SingleLoop => Some(current),
            _ => Some((current + self.items.len() - 1) % self.items.len()),
        }
    }

    fn next_shuffle(&mut self, current: usize) -> Option<usize> {
        if self.items.len() <= 1 {
            return Some(current);
        }
        if self.shuffle_order.is_empty() {
            self.rebuild_shuffle();
        }
        if self.shuffle_cursor >= self.shuffle_order.len() {
            self.rebuild_shuffle();
        }
        let index = self.shuffle_order.get(self.shuffle_cursor).copied()?;
        self.shuffle_cursor += 1;
        Some(index)
    }

    fn rebuild_shuffle(&mut self) {
        let len = self.items.len();
        let current = self.index;
        self.shuffle_order = (0..len).filter(|index| Some(*index) != current).collect();
        // Deterministic xorshift; sufficient for a local play queue.
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9e37_79b9_7f4a_7c15)
            | 1;
        for position in (1..self.shuffle_order.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let swap = (seed as usize) % (position + 1);
            self.shuffle_order.swap(position, swap);
        }
        self.shuffle_cursor = 0;
    }

    pub fn snapshot(&self) -> QueueSnapshotDto {
        QueueSnapshotDto {
            items: self.items.clone(),
            index: self.index.map(|index| index as f64),
            mode: self.mode.as_str().to_string(),
            muted: self.muted,
        }
    }

    pub fn save(&self) {
        let session = Session {
            items: self.items.clone(),
            index: self.index,
            mode: self.mode,
            muted: self.muted,
        };
        let Ok(json) = serde_json::to_vec_pretty(&session) else {
            return;
        };
        if let Some(parent) = self.session_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let temporary = self.session_file.with_extension("json.tmp");
        if std::fs::write(&temporary, json).is_ok() {
            let _ = std::fs::rename(&temporary, &self.session_file);
        }
    }
}

fn load_session(path: &Path) -> Option<Session> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Playback position persisted independently from the queue so that writing it
/// once per second never rewrites a 10k-item queue file (Mvp A1-05).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub path: String,
    pub position_ms: f64,
    pub playing: bool,
}

pub struct ProgressStore {
    file: PathBuf,
    last: Progress,
    last_write: Option<std::time::Instant>,
}

impl ProgressStore {
    pub fn new(file: PathBuf) -> Self {
        let last = std::fs::read(&file)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Progress>(&bytes).ok())
            .unwrap_or_default();
        Self {
            file,
            last,
            last_write: None,
        }
    }

    /// Returns the stored progress when it points at a non-empty path.
    pub fn load(&self) -> Option<Progress> {
        (!self.last.path.is_empty()).then(|| self.last.clone())
    }

    /// Records the live position. The file is rewritten when the track or the
    /// playing flag changes, at most once per second while playing, and on a
    /// paused seek (where the position jumps without playback advancing).
    pub fn update(&mut self, path: &str, position_ms: f64, playing: bool) {
        let position_ms = position_ms.max(0.0);
        let same_track = self.last.path == path && self.last.playing == playing;
        let seeked_while_paused =
            !playing && same_track && (position_ms - self.last.position_ms).abs() >= 500.0;
        let due = self
            .last_write
            .map(|instant| instant.elapsed() >= std::time::Duration::from_secs(1))
            .unwrap_or(true);
        self.last = Progress {
            path: path.to_string(),
            position_ms,
            playing,
        };
        if !same_track || seeked_while_paused || (playing && due) {
            self.save();
        }
    }

    fn save(&mut self) {
        let Ok(json) = serde_json::to_vec_pretty(&self.last) else {
            return;
        };
        if let Some(parent) = self.file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let temporary = self.file.with_extension("json.tmp");
        if std::fs::write(&temporary, json).is_ok()
            && std::fs::rename(&temporary, &self.file).is_ok()
        {
            self.last_write = Some(std::time::Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn item(title: &str) -> QueueItemDto {
        QueueItemDto {
            path: format!("/tmp/{title}.mp3"),
            title: title.to_string(),
            artist: None,
            album: None,
            duration_ms: Some(1000.0),
        }
    }

    fn queue() -> AudioQueue {
        let path = std::env::temp_dir().join(format!(
            "biu-queue-test-{}-{}.json",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        AudioQueue::new(path)
    }

    #[test]
    fn sequential_stops_at_end_and_loop_wraps() {
        let mut queue = queue();
        queue.replace(vec![item("a"), item("b")], 0, PlayMode::Sequential);
        assert_eq!(queue.next_index(), Some(1));
        queue.set_index(1);
        assert_eq!(queue.next_index(), None);

        queue.set_mode(PlayMode::ListLoop);
        assert_eq!(queue.next_index(), Some(0));
    }

    #[test]
    fn single_loop_keeps_current() {
        let mut queue = queue();
        queue.replace(vec![item("a"), item("b")], 0, PlayMode::SingleLoop);
        assert_eq!(queue.next_index(), Some(0));
        assert_eq!(queue.prev_index(), Some(0));
    }

    #[test]
    fn removing_current_adjusts_index() {
        let mut queue = queue();
        queue.replace(
            vec![item("a"), item("b"), item("c")],
            1,
            PlayMode::Sequential,
        );
        queue.remove(1);
        assert_eq!(queue.index(), Some(1));
        assert_eq!(queue.current().unwrap().title, "c");
    }

    #[test]
    fn moving_items_keeps_current_track() {
        let mut queue = queue();
        queue.replace(
            vec![item("a"), item("b"), item("c")],
            1,
            PlayMode::Sequential,
        );
        assert!(queue.move_item(0, 2));
        assert_eq!(queue.current().unwrap().title, "b");
        assert_eq!(queue.index(), Some(0));
        assert_eq!(queue.items()[0].title, "b");
        assert_eq!(queue.items()[2].title, "a");
        assert!(!queue.move_item(9, 0));
    }

    #[test]
    fn persists_queue_for_restart() {
        let mut queue = queue();
        queue.replace(vec![item("a"), item("b")], 1, PlayMode::Shuffle);
        let file = queue.session_file.clone();
        let restored = AudioQueue::new(file);
        assert_eq!(restored.items().len(), 2);
        assert_eq!(restored.index(), Some(1));
        assert_eq!(restored.mode(), PlayMode::Shuffle);
    }

    #[test]
    fn progress_store_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "biu-progress-test-{}-{}.json",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut store = ProgressStore::new(path.clone());
        assert!(store.load().is_none());
        store.update("/tmp/a.mp3", 12_500.0, true);
        let restored = ProgressStore::new(path.clone());
        let progress = restored.load().expect("progress written");
        assert_eq!(progress.path, "/tmp/a.mp3");
        assert_eq!(progress.position_ms, 12_500.0);
        assert!(progress.playing);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn progress_persists_paused_seek() {
        let path = std::env::temp_dir().join(format!(
            "biu-progress-seek-test-{}-{}.json",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut store = ProgressStore::new(path.clone());
        store.update("/tmp/a.mp3", 1_000.0, false);
        store.update("/tmp/a.mp3", 9_000.0, false);
        let progress = ProgressStore::new(path.clone())
            .load()
            .expect("seek written");
        assert_eq!(progress.position_ms, 9_000.0);
        assert!(!progress.playing);
        let _ = std::fs::remove_file(path);
    }
}
