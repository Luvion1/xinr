//! Region table: array of regions, indexed by region id.

use crate::RuntimeError;
use crate::xgc::region::{REGION_SIZE, Region};

/// Region table.
pub struct RegionTable {
    regions: [Region; Self::MAX_REGIONS],
    count: usize,
    next_alloc: usize,
}

impl RegionTable {
    /// Max regions in one table.
    pub const MAX_REGIONS: usize = 4096;

    /// Construct a table of `count` unbound regions.
    pub fn new(count: usize) -> Result<Self, RuntimeError> {
        if count > Self::MAX_REGIONS {
            return Err(RuntimeError::OutOfMemory);
        }
        let mut regions = [Region::unbound(); Self::MAX_REGIONS];
        for r in regions.iter_mut().take(count) {
            *r = Region {
                start: core::ptr::null_mut(),
                used: 0,
                capacity: REGION_SIZE,
                occupied: false,
            };
        }
        Ok(Self {
            regions,
            count,
            next_alloc: 0,
        })
    }

    /// Number of regions.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Read a region by index.
    pub fn get(&self, idx: usize) -> Option<&Region> {
        if idx < self.count {
            Some(&self.regions[idx])
        } else {
            None
        }
    }

    /// Mutable access by index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Region> {
        if idx < self.count {
            // SAFETY: idx is in bounds
            unsafe { Some(&mut *(&mut self.regions[idx] as *mut Region)) }
        } else {
            None
        }
    }

    /// Reserve the next free region slot. Returns region index.
    pub fn reserve_next(&mut self) -> Option<usize> {
        while self.next_alloc < self.count {
            let idx = self.next_alloc;
            self.next_alloc += 1;
            if let Some(r) = self.get_mut(idx)
                && r.is_available()
            {
                return Some(idx);
            }
        }
        None
    }

    /// Reset the allocation cursor.
    pub fn reset_cursor(&mut self) {
        self.next_alloc = 0;
    }

    /// Total addressable bytes.
    pub fn total_bytes(&self) -> usize {
        self.count * REGION_SIZE
    }
}
