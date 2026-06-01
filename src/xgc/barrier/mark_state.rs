//! Mark epoch: monotonically-increasing counter for GC cycles.

use core::sync::atomic::{AtomicU64, Ordering};

/// Tracks the current mark epoch.
pub struct MarkEpoch {
    current: AtomicU64,
}

impl MarkEpoch {
    /// Create a new epoch counter at zero.
    pub const fn new() -> Self {
        Self {
            current: AtomicU64::new(0),
        }
    }

    /// Current epoch value.
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Acquire)
    }

    /// Advance to the next epoch and return it.
    pub fn advance(&self) -> u64 {
        self.current.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Whether `epoch` is older than the current one.
    pub fn is_stale(&self, epoch: u64) -> bool {
        epoch < self.current.load(Ordering::Acquire)
    }
}

impl Default for MarkEpoch {
    fn default() -> Self {
        Self::new()
    }
}
