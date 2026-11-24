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

#[cfg(test)]
mod tests {
    use super::*;

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
            "biu-queue-test-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
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
    fn persists_queue_for_restart() {
        let mut queue = queue();
        queue.replace(vec![item("a"), item("b")], 1, PlayMode::Shuffle);
        let file = queue.session_file.clone();
        let restored = AudioQueue::new(file);
        assert_eq!(restored.items().len(), 2);
        assert_eq!(restored.index(), Some(1));
        assert_eq!(restored.mode(), PlayMode::Shuffle);
    }
}
