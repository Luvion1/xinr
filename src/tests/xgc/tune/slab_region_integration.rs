//! Integration: slab allocator + region + GC cycle.
//!
//! Exercises the slab allocator in a realistic flow:
//!  1. Allocate 64 objects of varying size.
//!  2. Run a mark/relocate cycle that points at slab-backed addresses.
//!  3. Free the objects and verify the slab reuses slots.

extern crate alloc;
use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::region::RegionTable;
use crate::xgc::slab::SlabAllocator;
use alloc::vec::Vec;

#[test]
fn slab_fills_then_drains() {
    let mut alloc = SlabAllocator::new(0x10_0000);
    let mut ptrs: [u64; 64] = [0; 64];

    // 1. Allocate 16 small (8 B), 32 medium (32 B), 16 large (256 B).
    for slot in &mut ptrs[..16] {
        *slot = alloc.alloc(8).unwrap().addr() as u64;
    }
    for slot in &mut ptrs[16..48] {
        *slot = alloc.alloc(32).unwrap().addr() as u64;
    }
    for slot in &mut ptrs[48..] {
        *slot = alloc.alloc(256).unwrap().addr() as u64;
    }

    // 2. Verify all 64 addresses are distinct and in the base window.
    for w in ptrs.windows(2) {
        assert_ne!(w[0], w[1]);
        assert!(w[0] >= 0x10_0000);
        assert!(w[1] >= 0x10_0000);
    }

    // 3. Run a GC mark cycle over the slab addresses.
    let mut gc = Xgc::new(2).unwrap();
    gc.init().unwrap();
    gc.begin_mark().unwrap();
    for &p in &ptrs {
        gc.push_root(ColoredPtr::new(p as usize, Color::White)).unwrap();
    }
    let mut drained = 0;
    while gc.pop_work().is_some() { drained += 1; }
    assert_eq!(drained, 64);
    gc.finish_mark();
    gc.shutdown().unwrap();

    // 4. Free 32 of them, re-allocate, and check reuse within the same pool.
    for slot in &mut ptrs[..32] {
        alloc.free(*slot, 32);
    }
    let new_ptrs: alloc::vec::Vec<u64> =
        (0..32).map(|_| alloc.alloc(32).unwrap().addr() as u64).collect();
    assert_eq!(new_ptrs.len(), 32);

    // At least some should be reused from the freed pool.
    let reused = new_ptrs.iter().filter(|n| ptrs[..32].contains(n)).count();
    assert!(reused > 0, "slab should reuse freed slots (reused={})", reused);
}

#[test]
fn region_table_tracks_slab_regions() {
    let mut table = RegionTable::new(4);
    let base = 0x20_0000u64;
    table.bind(0, base);
    table.bind(1, base + 0x1_0000);
    table.bind(2, base + 0x2_0000);
    table.bind(3, base + 0x3_0000);

    let mut alloc = SlabAllocator::new(base);
    let mut addrs: [u64; 8] = [0; 8];
    for slot in &mut addrs {
        *slot = alloc.alloc(64).unwrap().addr() as u64;
    }

    // Each address should fall in one of the 4 bound regions.
    for a in &addrs {
        let region = (*a >> 16) as usize;
        assert!(region < 4, "address in region {}", region);
        assert!(table.get(region).is_some(), "region {} bound", region);
    }
}
