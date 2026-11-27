use crate::db::Track;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Repeat {
    Off,
    #[default]
    All,
    One,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Phase {
    #[default]
    Idle,
    Loading,
    Playing,
    Paused,
    Waiting,
    Failed,
}
#[derive(Default)]
pub struct Player {
    pub queue: Vec<Track>,
    pub index: Option<usize>,
    pub order: Vec<usize>,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub karaoke: bool,
    pub phase: Phase,
    pub generation: u64,
    pub error: String,
}
impl Player {
    pub fn current(&self) -> Option<&Track> {
        self.index.and_then(|i| self.queue.get(i))
    }
    pub fn start(&mut self, tracks: Vec<Track>, index: usize) -> bool {
        if tracks.is_empty() || index >= tracks.len() {
            return false;
        }
        self.queue = tracks;
        self.index = Some(index);
        self.reorder();
        self.load(index);
        true
    }
    fn load(&mut self, index: usize) {
        self.index = Some(index);
        self.generation += 1;
        self.phase = Phase::Loading;
        self.error.clear();
    }
    pub fn reload(&mut self) -> bool {
        if let Some(i) = self.index {
            self.load(i);
            true
        } else {
            false
        }
    }
    pub fn reorder(&mut self) {
        self.order = (0..self.queue.len()).collect();
        if self.shuffle && !self.karaoke {
            self.order.shuffle(&mut rand::rng());
            if let Some(at) = self
                .index
                .and_then(|i| self.order.iter().position(|n| *n == i))
            {
                self.order.swap(0, at);
            }
        }
    }
    pub fn enqueue(&mut self, track: Track) -> bool {
        self.queue.push(track);
        self.order.push(self.queue.len() - 1);
        if self.index.is_none() || (self.karaoke && self.phase == Phase::Waiting) {
            self.load(self.queue.len() - 1);
            true
        } else {
            false
        }
    }
    pub fn next(&mut self, natural: bool, failure: bool) -> bool {
        if natural && self.repeat == Repeat::One && !self.karaoke && !failure {
            return self.reload();
        }
        let next = self
            .index
            .and_then(|i| self.order.iter().position(|n| *n == i))
            .and_then(|p| self.order.get(p + 1).copied());
        if let Some(i) = next {
            self.load(i);
            true
        } else if !failure && !self.karaoke && self.repeat == Repeat::All && !self.queue.is_empty()
        {
            self.load(self.order[0]);
            true
        } else {
            self.phase = if failure {
                Phase::Failed
            } else if self.karaoke {
                Phase::Waiting
            } else {
                Phase::Idle
            };
            false
        }
    }
    pub fn previous(&mut self) -> bool {
        if let Some(at) = self
            .index
            .and_then(|i| self.order.iter().position(|n| *n == i))
            && at > 0
        {
            self.load(self.order[at - 1]);
            return true;
        }
        self.reload()
    }
    pub fn replace(&mut self, track: Track) -> bool {
        if let Some(i) = self.index {
            self.queue[i] = track;
            self.load(i);
            true
        } else {
            self.enqueue(track)
        }
    }
    pub fn prioritize(&mut self, i: usize) {
        if let Some(current) = self.index
            && i > current
            && i < self.queue.len()
        {
            let t = self.queue.remove(i);
            self.queue.insert(current + 1, t);
            self.reorder();
        }
    }
    pub fn remove(&mut self, i: usize) -> bool {
        if i >= self.queue.len() {
            return false;
        }
        let current = self.index;
        self.queue.remove(i);
        self.index = match current {
            Some(_) if self.queue.is_empty() => None,
            Some(n) if i < n => Some(n - 1),
            Some(n) => Some(n.min(self.queue.len() - 1)),
            None => None,
        };
        self.reorder();
        if self.queue.is_empty() {
            self.generation += 1;
            self.phase = Phase::Idle;
            return true;
        }
        if current == Some(i) {
            self.reload()
        } else {
            false
        }
    }
    pub fn accept(&mut self, token: u64) -> bool {
        if token != self.generation {
            return false;
        }
        self.phase = Phase::Playing;
        true
    }
    pub fn clear(&mut self) {
        self.queue.clear();
        self.order.clear();
        self.index = None;
        self.phase = Phase::Idle;
        self.generation += 1;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn track(id: i64) -> Track {
        Track {
            id,
            path: format!("{id}.wav"),
            title: id.to_string(),
            artist: String::new(),
            album: String::new(),
            duration: 1.0,
            cover_path: None,
            source: "local".into(),
            url: None,
            liked: false,
        }
    }
    #[test]
    fn queue_karaoke_replacement_failure_and_cancellation() {
        let mut p = Player {
            karaoke: true,
            ..Default::default()
        };
        assert!(p.start((1..=4).map(track).collect(), 0));
        let old = p.generation;
        p.prioritize(3);
        assert_eq!(p.queue[1].id, 4);
        p.remove(2);
        assert_eq!(p.queue.len(), 3);
        p.replace(track(9));
        assert_eq!(p.queue[1].id, 4);
        assert!(!p.accept(old));
        assert!(p.next(false, false));
        assert_eq!(p.current().unwrap().id, 4);
        assert!(p.next(true, false));
        assert!(!p.next(true, false));
        assert_eq!(p.phase, Phase::Waiting);
        assert!(p.enqueue(track(8)));
        assert_eq!(p.current().unwrap().id, 8);
        assert!(!p.next(false, true));
        assert_eq!(p.phase, Phase::Failed);
        p.karaoke = false;
        assert!(p.start(vec![track(1), track(2)], 0));
        assert!(p.next(false, true));
        assert!(!p.next(false, true));
        assert_eq!(p.phase, Phase::Failed);
        p.clear();
        assert!(p.current().is_none());
    }
}
