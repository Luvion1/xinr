//! Allocation profiler.

pub mod site;
pub mod stats;

use crate::xgc::profile::site::SiteId;
use crate::xgc::profile::stats::SiteStats;

/// Allocation profiler.
#[allow(dead_code)]
pub struct AllocProfiler {
    sites: [Option<SiteStats>; 64],
    count: usize,
}

impl AllocProfiler {
    /// Construct an empty profiler.
    pub const fn new() -> Self {
        Self {
            sites: [None; 64],
            count: 0,
        }
    }

    /// Record an allocation.
    pub fn record_alloc(&mut self, id: SiteId, bytes: u64) {
        let idx = (id.0 as usize) % self.sites.len();
        let entry = self.sites[idx].get_or_insert(SiteStats::new());
        entry.record_alloc(bytes);
    }

    /// Record a free.
    pub fn record_free(&mut self, id: SiteId, bytes: u64) {
        let idx = (id.0 as usize) % self.sites.len();
        if let Some(s) = self.sites[idx].as_mut() {
            s.record_free(bytes);
        }
    }

    /// Read site stats.
    pub fn get(&self, id: SiteId) -> Option<&SiteStats> {
        let idx = (id.0 as usize) % self.sites.len();
        self.sites[idx].as_ref()
    }

    /// Total bytes allocated across all sites.
    pub fn total_alloc_bytes(&self) -> u64 {
        self.sites
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.alloc_bytes)
            .sum()
    }

    /// Number of distinct sites.
    pub fn site_count(&self) -> usize {
        self.sites.iter().filter(|s| s.is_some()).count()
    }

    /// Reset all stats (zero counters; preserve site entries).
    pub fn reset(&mut self) {
        for s in self.sites.iter_mut() {
            if let Some(entry) = s.as_mut() {
                *entry = SiteStats::new();
            }
        }
    }
}

impl Default for AllocProfiler {
    fn default() -> Self {
        Self::new()
    }
}
