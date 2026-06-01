//! Region descriptor: lightweight, copyable per-region metadata.

use crate::xgc::region::REGION_SIZE;

/// Per-region descriptor (4 × 8 bytes).
#[derive(Debug, Clone, Copy)]
pub struct RegionDescriptor {
    /// Base address of the region.
    pub base: u64,
    /// Bytes currently allocated.
    pub used: u32,
    /// Region state packed into a u32 (0=free, 1=used, 2=full).
    pub state: u32,
}

impl RegionDescriptor {
    /// Construct a free descriptor.
    pub const fn empty(base: u64) -> Self {
        Self {
            base,
            used: 0,
            state: 0,
        }
    }

    /// Capacity in bytes (always `REGION_SIZE`).
    pub const fn capacity(&self) -> u32 {
        REGION_SIZE as u32
    }

    /// Free bytes remaining.
    pub const fn free(&self) -> u32 {
        REGION_SIZE as u32 - self.used
    }
}
