//! Cycle detection for the GC.

pub mod queue;
pub mod stats;

use crate::xgc::colored::ColoredPtr;
use crate::xgc::cycle::queue::CycleQueue;
use crate::xgc::cycle::stats::{CycleCandidate, CycleStats};
use crate::xgc::finalize::weak::WeakTable;

/// Cycle detector.
pub struct CycleDetector {
    queue: CycleQueue,
    stats: CycleStats,
    /// Above this pending count, switch from linear to DFS scan.
    pub dfs_threshold: usize,
}

impl CycleDetector {
    /// Construct an empty detector.
    pub const fn new() -> Self {
        Self {
            queue: CycleQueue::new(),
            stats: CycleStats::new(),
            dfs_threshold: 8,
        }
    }

    /// Submit one candidate.
    pub fn submit(&mut self, c: CycleCandidate) {
        let _ = self.queue.enqueue(c);
    }

    /// Run a single pass against the weak table.
    pub fn run_pass(&mut self, wt: &WeakTable) -> CycleStats {
        let mut stats = self.stats;
        stats.reclaimed = 0;
        while let Some(c) = self.queue.dequeue() {
            if !wt.contains(crate::xgc::finalize::weak::WeakRef(c.weak_ref)) {
                stats.reclaimed += 1;
            }
            stats.candidates += 1;
        }
        stats
    }

    /// Number of pending candidates.
    pub fn pending(&self) -> usize {
        self.queue.count
    }

    /// Whether the pending count is over the DFS threshold.
    pub fn should_dfs(&self) -> bool {
        self.pending() >= self.dfs_threshold
    }

    /// Reset statistics.
    pub fn reset_stats(&mut self) {
        self.stats = CycleStats::new();
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
fn _phantom(_: ColoredPtr) {}
