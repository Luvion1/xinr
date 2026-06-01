//! Thread-local context: per-thread buffers and statistics.

use crate::xgc::colored::ColoredPtr;
use crate::xgc::tl::buffer::Tlb;

/// Per-thread GC context.
pub struct ThreadCtx {
    pub id: u32,
    pub tlb: Tlb,
    pub allocs: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ThreadCtx {
    /// Construct a context for a given thread id.
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            tlb: Tlb::new(),
            allocs: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Try to satisfy an allocation from the local buffer. Returns None
    /// if the buffer is empty (caller should fall back to the heap).
    pub fn try_local_alloc(&mut self) -> Option<ColoredPtr> {
        let p = self.tlb.pop()?;
        self.cache_hits += 1;
        Some(p)
    }

    /// Return a pointer to the local buffer (e.g. after collection).
    pub fn return_to_local(&mut self, p: ColoredPtr) {
        if !self.tlb.push(p) {
            self.cache_misses += 1;
        }
    }

    /// Record a fallback allocation.
    pub fn record_alloc(&mut self) {
        self.allocs += 1;
        self.cache_misses += 1;
    }

    /// Hit rate as 0.0..=1.0.
    pub fn hit_rate(&self) -> f32 {
        if self.allocs == 0 {
            0.0
        } else {
            self.cache_hits as f32 / self.allocs as f32
        }
    }
}
