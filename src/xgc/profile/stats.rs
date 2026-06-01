//! Per-site allocation stats.

/// Stats for one allocation site.
#[derive(Debug, Clone, Copy)]
pub struct SiteStats {
    /// Number of allocations.
    pub alloc_count: u64,
    /// Total bytes allocated.
    pub alloc_bytes: u64,
    /// Total bytes freed.
    pub free_bytes: u64,
}

impl SiteStats {
    /// Construct zeroed stats.
    pub const fn new() -> Self {
        Self {
            alloc_count: 0,
            alloc_bytes: 0,
            free_bytes: 0,
        }
    }

    /// Record one allocation.
    pub fn record_alloc(&mut self, bytes: u64) {
        self.alloc_count += 1;
        self.alloc_bytes += bytes;
    }

    /// Record one free.
    pub fn record_free(&mut self, bytes: u64) {
        self.free_bytes += bytes;
    }

    /// Live bytes (alloc - free).
    pub fn live_bytes(&self) -> u64 {
        self.alloc_bytes.saturating_sub(self.free_bytes)
    }
}

impl Default for SiteStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-site entry for the profiler (alias for `SiteStats`).
pub type SiteEntry = SiteStats;
