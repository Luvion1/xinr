//! Slab allocation lifecycle tests.

use crate::xgc::slab::SlabAllocator;

#[test]
fn alloc_returns_multiple_unique_slots() {
    let mut slab = SlabAllocator::new(0x10000);
    let a = slab.alloc(48).map(|p| p.addr()).unwrap();
    let b = slab.alloc(48).map(|p| p.addr()).unwrap();
    let c = slab.alloc(48).map(|p| p.addr()).unwrap();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn free_returns_slot_to_pool() {
    let mut slab = SlabAllocator::new(0x10000);
    let addr = slab.alloc(32).map(|p| p.addr()).unwrap();
    slab.free(addr as u64, 32);
    assert_eq!(slab.slab_count(), 1);
}

#[test]
fn max_slabs_limit_enforced() {
    let mut slab = SlabAllocator::new(0x10000);
    for _ in 0..1000 {
        slab.alloc(8);
    }
    assert!(slab.slab_count() <= 32);
}