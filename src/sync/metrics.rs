//! Metrics: process-wide counters for monitoring.

use core::sync::atomic::{AtomicU64, Ordering};

/// Process-wide metrics. Each counter is an `AtomicU64` for thread-safe
/// increment without locks.
pub struct Metrics {
    pub allocs: AtomicU64,
    pub frees: AtomicU64,
    pub marks: AtomicU64,
    pub sweeps: AtomicU64,
    pub evictions: AtomicU64,
    pub triggers: AtomicU64,
    pub cycles: AtomicU64,
    pub errors: AtomicU64,
}

impl Metrics {
    /// Construct a new metrics block, all counters zero.
    pub const fn new() -> Self {
        Self {
            allocs: AtomicU64::new(0),
            frees: AtomicU64::new(0),
            marks: AtomicU64::new(0),
            sweeps: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            triggers: AtomicU64::new(0),
            cycles: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    /// Increment the allocs counter.
    pub fn inc_alloc(&self) {
        self.allocs.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the frees counter.
    pub fn inc_free(&self) {
        self.frees.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the marks counter.
    pub fn inc_mark(&self) {
        self.marks.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the sweeps counter.
    pub fn inc_sweep(&self) {
        self.sweeps.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the evictions counter.
    pub fn inc_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the triggers counter.
    pub fn inc_trigger(&self) {
        self.triggers.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the cycles counter.
    pub fn inc_cycle(&self) {
        self.cycles.fetch_add(1, Ordering::Relaxed);
    }
    /// Increment the errors counter.
    pub fn inc_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot all counters into an array.
    pub fn snapshot(&self) -> [u64; 8] {
        [
            self.allocs.load(Ordering::Relaxed),
            self.frees.load(Ordering::Relaxed),
            self.marks.load(Ordering::Relaxed),
            self.sweeps.load(Ordering::Relaxed),
            self.evictions.load(Ordering::Relaxed),
            self.triggers.load(Ordering::Relaxed),
            self.cycles.load(Ordering::Relaxed),
            self.errors.load(Ordering::Relaxed),
        ]
    }

    /// Number of live objects: `allocs - frees` (saturating).
    pub fn live(&self) -> u64 {
        self.allocs
            .load(Ordering::Relaxed)
            .saturating_sub(self.frees.load(Ordering::Relaxed))
    }

    /// Mark progress ratio: `marks / max(1, cycles)`.
    pub fn marks_per_cycle(&self) -> u64 {
        let c = self.cycles.load(Ordering::Relaxed);
        let m = self.marks.load(Ordering::Relaxed);
        m / c.max(1)
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.allocs.store(0, Ordering::Relaxed);
        self.frees.store(0, Ordering::Relaxed);
        self.marks.store(0, Ordering::Relaxed);
        self.sweeps.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.triggers.store(0, Ordering::Relaxed);
        self.cycles.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
    }

    /// Add `delta` to a specific counter (0..=7).
    pub fn add(&self, idx: usize, delta: u64) {
        let c = match idx {
            0 => &self.allocs,
            1 => &self.frees,
            2 => &self.marks,
            3 => &self.sweeps,
            4 => &self.evictions,
            5 => &self.triggers,
            6 => &self.cycles,
            7 => &self.errors,
            _ => return,
        };
        c.fetch_add(delta, Ordering::Relaxed);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_is_allocs_minus_frees() {
        let m = Metrics::new();
        m.inc_alloc();
        m.inc_alloc();
        m.inc_alloc();
        m.inc_free();
        assert_eq!(m.live(), 2);
    }

    #[test]
    fn live_saturates_on_underflow() {
        let m = Metrics::new();
        m.inc_free();
        m.inc_free();
        assert_eq!(m.live(), 0);
    }

    #[test]
    fn marks_per_cycle_uses_max1() {
        let m = Metrics::new();
        for _ in 0..4 {
            m.inc_mark();
        }
        assert_eq!(m.marks_per_cycle(), 4);
    }

    #[test]
    fn reset_zeros_everything() {
        let m = Metrics::new();
        m.inc_alloc();
        m.inc_free();
        m.inc_mark();
        m.inc_sweep();
        m.inc_eviction();
        m.inc_trigger();
        m.inc_cycle();
        m.inc_error();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s, [0; 8]);
    }

    #[test]
    fn add_dispatches_by_index() {
        let m = Metrics::new();
        m.add(0, 5);
        m.add(3, 2);
        m.add(99, 1);
        let s = m.snapshot();
        assert_eq!(s[0], 5);
        assert_eq!(s[3], 2);
    }
}
