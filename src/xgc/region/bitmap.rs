//! Mark bitmap for a single region.

use crate::xgc::region::REGION_SIZE;

/// One bit per 8-byte slot.
pub const BITMAP_BYTES: usize = REGION_SIZE / 8 / 8;

/// Mark bitmap. One bit per 8-byte slot.
pub struct MarkBitmap {
    bits: [u64; BITMAP_BYTES / 8],
    count: usize,
}

impl MarkBitmap {
    /// Construct a clear bitmap.
    pub const fn new() -> Self {
        Self {
            bits: [0; BITMAP_BYTES / 8],
            count: BITMAP_BYTES / 8,
        }
    }

    /// Mark slot `offset` as live.
    pub fn mark(&mut self, offset: usize) {
        if offset >= self.count {
            return;
        }
        let word = offset / 64;
        let bit = offset % 64;
        if word < self.bits.len() {
            self.bits[word] |= 1u64 << bit;
        }
    }

    /// Unmark slot `offset`.
    pub fn unmark(&mut self, offset: usize) {
        if offset >= self.count {
            return;
        }
        let word = offset / 64;
        let bit = offset % 64;
        if word < self.bits.len() {
            self.bits[word] &= !(1u64 << bit);
        }
    }

    /// Whether `offset` is marked.
    pub fn is_marked(&self, offset: usize) -> bool {
        if offset >= self.count {
            return false;
        }
        let word = offset / 64;
        let bit = offset % 64;
        word < self.bits.len() && (self.bits[word] & (1u64 << bit)) != 0
    }

    /// Clear all bits.
    pub fn clear(&mut self) {
        for w in self.bits.iter_mut() {
            *w = 0;
        }
    }

    /// Visit each marked offset.
    pub fn for_each_marked<F: FnMut(usize)>(&self, mut f: F) {
        for (w_idx, &word) in self.bits.iter().enumerate() {
            if word == 0 {
                continue;
            }
            for bit in 0..64 {
                if (word >> bit) & 1 == 1 {
                    f(w_idx * 64 + bit);
                }
            }
        }
    }
}

impl Default for MarkBitmap {
    fn default() -> Self {
        Self::new()
    }
}
