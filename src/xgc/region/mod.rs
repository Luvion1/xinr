//! Region subsystem: region descriptors, allocation table, mark bitmap.

pub mod bitmap;
pub mod descriptor;
pub mod table;

/// Fixed size of a single region in bytes.
pub const REGION_SIZE: usize = 1 << 20; // 1 MiB

/// Region metadata. Pinned (never moved by GC).
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub start: *mut u8,
    pub used: usize,
    pub capacity: usize,
    pub occupied: bool,
}

impl Region {
    /// Create an unbound region placeholder.
    pub const fn unbound() -> Self {
        Self {
            start: core::ptr::null_mut(),
            used: 0,
            capacity: REGION_SIZE,
            occupied: false,
        }
    }

    /// Whether the region can accept new allocations.
    pub fn is_available(&self) -> bool {
        !self.occupied && !self.start.is_null()
    }

    /// Remaining free bytes within this region.
    pub fn free(&self) -> usize {
        self.capacity.saturating_sub(self.used)
    }
}
