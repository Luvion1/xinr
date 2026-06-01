//! GC statistics: counters and timing for observability.
//!
//! These are monotonically increasing counters that can be sampled at any
//! time. They are not strictly atomic, so sampling may see slightly stale
//! values during a concurrent cycle.

use crate::xgc::colored::ColoredPtr;

/// Aggregate statistics across all GC cycles.
#[derive(Debug, Clone, Copy, Default)]
pub struct GcStats {
    /// Total bytes ever allocated.
    pub bytes_allocated: u64,
    /// Total bytes freed by the last sweep.
    pub bytes_freed: u64,
    /// Cumulative cycles completed.
    pub cycles: u64,
    /// Cumulative objects marked.
    pub objects_marked: u64,
    /// Cumulative objects relocated.
    pub objects_relocated: u64,
    /// Cumulative SATB barrier invocations.
    pub satb_invocations: u64,
    /// Cumulative ref-update invocations.
    pub ref_update_invocations: u64,
    /// Peak live bytes since process start.
    pub peak_live_bytes: u64,
}

impl GcStats {
    /// Construct zeroed stats.
    pub const fn new() -> Self {
        Self {
            bytes_allocated: 0,
            bytes_freed: 0,
            cycles: 0,
            objects_marked: 0,
            objects_relocated: 0,
            satb_invocations: 0,
            ref_update_invocations: 0,
            peak_live_bytes: 0,
        }
    }

    /// Update peak live bytes if `current` is greater.
    pub fn update_peak(&mut self, current: u64) {
        if current > self.peak_live_bytes {
            self.peak_live_bytes = current;
        }
    }

    /// Record an allocation.
    pub fn record_alloc(&mut self, bytes: u64) {
        self.bytes_allocated = self.bytes_allocated.saturating_add(bytes);
    }

    /// Record a free.
    pub fn record_free(&mut self, bytes: u64) {
        self.bytes_freed = self.bytes_freed.saturating_add(bytes);
    }

    /// Net live bytes (allocated - freed).
    pub fn live_bytes(&self) -> i64 {
        self.bytes_allocated as i64 - self.bytes_freed as i64
    }
}

/// Helper to format stats as a debug string.
pub fn format_stats(stats: &GcStats) -> ColoredPtr {
    // Placeholder; the colored pointer is a type with `Display`.
    // Real formatting would use a writer.
    ColoredPtr::new(
        stats.bytes_allocated as usize,
        crate::xgc::colored::Color::White,
    )
}
