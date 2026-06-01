//! Pressure threshold: tracks bytes allocated since the last GC cycle.
//!
//! When allocations cross a configurable fraction of the heap, the runtime
//! schedules a new mark cycle.

/// Pressure threshold configuration.
#[derive(Debug, Clone, Copy)]
pub struct PressureConfig {
    /// Total heap size in bytes.
    pub heap_bytes: u64,
    /// Trigger when allocated bytes cross this fraction of heap_bytes.
    pub trigger_ratio: u32,
    /// Minimum interval (cycles) between triggers.
    pub min_interval: u32,
}

impl PressureConfig {
    /// Default: trigger at 80% full, no interval guard.
    pub const fn default_for(heap_bytes: u64) -> Self {
        Self {
            heap_bytes,
            trigger_ratio: 80,
            min_interval: 0,
        }
    }

    /// Bytes at which the trigger fires.
    pub const fn trigger_bytes(&self) -> u64 {
        (self.heap_bytes * self.trigger_ratio as u64) / 100
    }
}

/// Allocation counter. Monotonic between cycles.
#[derive(Debug, Clone, Copy, Default)]
pub struct PressureMeter {
    pub allocated: u64,
    pub freed: u64,
    pub cycles: u32,
}

impl PressureMeter {
    /// Create a fresh meter.
    pub const fn new() -> Self {
        Self {
            allocated: 0,
            freed: 0,
            cycles: 0,
        }
    }

    /// Record an allocation.
    pub fn record_alloc(&mut self, bytes: u64) {
        self.allocated = self.allocated.saturating_add(bytes);
    }

    /// Record a free (sweep) event.
    pub fn record_free(&mut self, bytes: u64) {
        self.freed = self.freed.saturating_add(bytes);
    }

    /// Mark the end of a GC cycle.
    pub fn end_cycle(&mut self) {
        self.cycles = self.cycles.saturating_add(1);
        // Reset allocation counter; freed is cumulative.
        self.allocated = 0;
    }

    /// Live bytes (allocated - freed).
    pub fn live(&self) -> i64 {
        self.allocated as i64 - self.freed as i64
    }
}
