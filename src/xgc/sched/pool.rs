//! Worker pool: array of `WorkDeque`s, one per GC worker.

use crate::xgc::colored::ColoredPtr;
use crate::xgc::sched::deque::WorkDeque;

const MAX_WORKERS: usize = 8;

/// Multi-worker task pool with work stealing.
pub struct WorkerPool {
    deques: [WorkDeque; MAX_WORKERS],
    active: u8,
}

impl WorkerPool {
    /// Construct an empty pool.
    pub const fn new() -> Self {
        Self {
            deques: [const { WorkDeque::new() }; MAX_WORKERS],
            active: 0,
        }
    }

    /// Register a worker.
    pub fn register(&mut self) -> Option<u8> {
        if (self.active as usize) >= MAX_WORKERS {
            return None;
        }
        let id = self.active;
        self.active += 1;
        Some(id)
    }

    /// Local push to a worker's own deque.
    pub fn local_push(&mut self, id: u8, p: ColoredPtr) -> bool {
        self.deques[id as usize].push(p)
    }

    /// Local pop from a worker's own deque.
    pub fn local_pop(&mut self, id: u8) -> Option<ColoredPtr> {
        self.dequeues_work(id, |d| d.pop())
    }

    /// Steal the oldest item from another worker.
    pub fn steal(&mut self, thief: u8) -> Option<ColoredPtr> {
        for i in 0..self.active as usize {
            if i as u8 == thief {
                continue;
            }
            if let Some(p) = self.deques[i].steal() {
                return Some(p);
            }
        }
        None
    }

    /// Total pending items across all deques.
    pub fn total_pending(&self) -> usize {
        self.deques[..self.active as usize]
            .iter()
            .map(|d| d.len())
            .sum()
    }

    /// Number of active workers.
    pub fn worker_count(&self) -> u8 {
        self.active
    }

    fn dequeues_work<F: FnOnce(&mut WorkDeque) -> Option<ColoredPtr>>(
        &mut self,
        id: u8,
        f: F,
    ) -> Option<ColoredPtr> {
        f(&mut self.deques[id as usize])
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}
