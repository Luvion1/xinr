//! Slab allocator: pool of fixed-size slabs, one per `SlabSize`.

pub mod page;
pub mod size;

use crate::xgc::colored::ColoredPtr;
use crate::xgc::slab::page::Slab;
use crate::xgc::slab::size::SlabSize;

const MAX_SLABS: usize = 32;

/// Per-size slab pool.
#[allow(dead_code)]
struct SizePool {
    size: SlabSize,
    partial: [Option<usize>; MAX_SLABS],
    partial_head: usize,
    full: [Option<usize>; MAX_SLABS],
    full_head: usize,
    next_slot: u32,
}

impl SizePool {
    const fn new(size: SlabSize) -> Self {
        Self {
            size,
            partial: [None; MAX_SLABS],
            partial_head: 0,
            full: [None; MAX_SLABS],
            full_head: 0,
            next_slot: 0,
        }
    }

    fn add_partial(&mut self, idx: usize) {
        if self.partial_head < self.partial.len() {
            self.partial[self.partial_head] = Some(idx);
            self.partial_head += 1;
        }
    }
}

/// Slab allocator.
pub struct SlabAllocator {
    pools: [SizePool; 6],
    slabs: [Option<Slab>; MAX_SLABS],
    slab_count: usize,
    base: u64,
}

impl SlabAllocator {
    /// Construct a slab allocator with `base` as the first address.
    pub const fn new(base: u64) -> Self {
        Self {
            pools: [
                SizePool::new(SlabSize::S16),
                SizePool::new(SlabSize::S32),
                SizePool::new(SlabSize::S64),
                SizePool::new(SlabSize::S128),
                SizePool::new(SlabSize::S256),
                SizePool::new(SlabSize::S512),
            ],
            slabs: [const { None }; MAX_SLABS],
            slab_count: 0,
            base,
        }
    }

    /// Allocate from the smallest slab that fits `bytes`.
    pub fn alloc(&mut self, bytes: usize) -> Option<ColoredPtr> {
        let size = SlabSize::for_bytes(bytes);
        let pool_idx = Self::pool_index(size)?;
        // Grow if needed.
        if self.pools[pool_idx].partial_head == 0 && self.slab_count < self.slabs.len() {
            let base = self.base + (self.slab_count as u64) * 4096;
            let mut slab = Slab::new(size, base);
            slab.init_full();
            self.slabs[self.slab_count] = Some(slab);
            self.pools[pool_idx].add_partial(self.slab_count);
            self.slab_count += 1;
        }
        // Pull from partial.
        if self.pools[pool_idx].partial_head > 0 {
            self.pools[pool_idx].partial_head -= 1;
            let idx = self.pools[pool_idx].partial[self.pools[pool_idx].partial_head].take()?;
            let slab = self.slabs[idx].as_mut()?;
            let slot = slab.alloc()?;
            if !slab.has_free() && self.pools[pool_idx].full_head < self.pools[pool_idx].full.len()
            {
                self.pools[pool_idx].full[self.pools[pool_idx].full_head] = Some(idx);
                self.pools[pool_idx].full_head += 1;
            } else {
                self.pools[pool_idx].add_partial(idx);
            }
            return Some(ColoredPtr::new(
                (slab.base() + slot as u64 * size.0 as u64) as usize,
                crate::xgc::colored::Color::White,
            ));
        }
        None
    }

    /// Free a slot at address `addr` of size `bytes`.
    pub fn free(&mut self, addr: u64, bytes: usize) {
        let size = SlabSize::for_bytes(bytes);
        let Some(_pool_idx) = Self::pool_index(size) else {
            return;
        };
        let base = self.base;
        if addr < base {
            return;
        }
        let off = addr - base;
        let page_idx = (off / 4096) as usize;
        let slot = ((off % 4096) / size.0 as u64) as u32;
        if page_idx >= self.slab_count {
            return;
        }
        if let Some(slab) = self.slabs[page_idx].as_mut() {
            slab.free(slot);
        }
    }

    /// Number of allocated slabs.
    pub fn slab_count(&self) -> usize {
        self.slab_count
    }

    /// Total bytes reserved.
    pub fn reserved(&self) -> u64 {
        self.slab_count as u64 * 4096
    }

    fn pool_index(size: SlabSize) -> Option<usize> {
        match size {
            SlabSize::S16 => Some(0),
            SlabSize::S32 => Some(1),
            SlabSize::S64 => Some(2),
            SlabSize::S128 => Some(3),
            SlabSize::S256 => Some(4),
            SlabSize::S512 => Some(5),
            _ => None,
        }
    }
}
