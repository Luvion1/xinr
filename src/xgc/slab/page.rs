//! A single slab: a page containing a fixed number of free slots.

use crate::xgc::slab::size::SlabSize;

/// Slab header.
pub struct Slab {
    size: SlabSize,
    bitmap: [u64; 8],
    next: Option<usize>,
    free_count: u32,
    base: u64,
}

impl Slab {
    /// Construct a slab for `size` at `base` address.
    pub fn new(size: SlabSize, base: u64) -> Self {
        Self {
            size,
            bitmap: [0u64; 8],
            next: None,
            free_count: 0,
            base,
        }
    }

    /// Initialize the free list to "all free".
    pub fn init_full(&mut self) {
        self.free_count = self.size.slots_per_page();
        for w in self.bitmap.iter_mut() {
            *w = u64::MAX;
        }
    }

    /// Allocate a slot. Returns the slot index, or None if full.
    pub fn alloc(&mut self) -> Option<u32> {
        if self.free_count == 0 {
            return None;
        }
        for w in 0..self.bitmap.len() {
            if self.bitmap[w] != 0 {
                let bit = self.bitmap[w].trailing_zeros();
                self.bitmap[w] &= !(1u64 << bit);
                self.free_count -= 1;
                return Some((w as u32) * 64 + bit);
            }
        }
        None
    }

    /// Free a slot by index.
    pub fn free(&mut self, slot: u32) {
        let w = (slot / 64) as usize;
        let b = slot % 64;
        if (self.bitmap[w] & (1u64 << b)) == 0 {
            self.bitmap[w] |= 1u64 << b;
            self.free_count += 1;
        }
    }

    /// Whether the slab has any free slots.
    pub fn has_free(&self) -> bool {
        self.free_count > 0
    }

    /// Number of free slots.
    pub fn free_count(&self) -> u32 {
        self.free_count
    }

    /// Number of used slots.
    pub fn used(&self) -> u32 {
        self.size.slots_per_page() - self.free_count
    }

    /// Total slot count.
    pub fn total(&self) -> u32 {
        self.size.slots_per_page()
    }

    /// Base address of the underlying memory.
    pub fn base(&self) -> u64 {
        self.base
    }

    /// Slab size.
    pub fn size(&self) -> SlabSize {
        self.size
    }

    /// Next slab in the partial list.
    pub fn next(&self) -> Option<usize> {
        self.next
    }

    /// Set the next pointer.
    pub fn set_next(&mut self, n: Option<usize>) {
        self.next = n;
    }
}
