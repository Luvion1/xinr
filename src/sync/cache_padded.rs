//! Cache-padded: pad a value to a cache-line boundary to prevent false sharing.

use core::sync::atomic::{AtomicU64, Ordering};

/// Standard cache line size in bytes (most x86/ARM CPUs).
pub const CACHE_LINE: usize = 64;

/// Pad a value to fill a 64-byte cache line.
#[repr(align(64))]
pub struct CachePadded<T> {
    pub value: T,
}

impl<T> CachePadded<T> {
    /// Wrap a value with cache-line padding.
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

/// A 64-byte aligned atomic counter, suitable for use as a per-thread
/// statistic without false sharing.
#[repr(align(64))]
pub struct PaddedCounter {
    counter: AtomicU64,
    _pad: [u8; CACHE_LINE - 8],
}

impl PaddedCounter {
    /// Construct a zero counter.
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            _pad: [0; CACHE_LINE - 8],
        }
    }

    /// Increment by 1.
    pub fn inc(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
    /// Add `n`.
    pub fn add(&self, n: u64) {
        self.counter.fetch_add(n, Ordering::Relaxed);
    }
    /// Load.
    pub fn load(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

impl Default for PaddedCounter {
    fn default() -> Self {
        Self::new()
    }
}
