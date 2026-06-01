//! Slab and scheduler tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::sched::deque::WorkDeque;
use crate::xgc::sched::pool::WorkerPool;
use crate::xgc::slab::SlabAllocator;
use crate::xgc::slab::size::SlabSize;

// --- SlabSize ---

#[test]
fn slab_size_pick() {
    assert_eq!(SlabSize::for_bytes(8), SlabSize::S16);
    assert_eq!(SlabSize::for_bytes(20), SlabSize::S32);
    assert_eq!(SlabSize::for_bytes(70), SlabSize::S128);
    assert_eq!(SlabSize::for_bytes(300), SlabSize::S512);
}

#[test]
fn slab_size_slots_per_page() {
    assert_eq!(SlabSize::S16.slots_per_page(), 256);
    assert_eq!(SlabSize::S32.slots_per_page(), 128);
    assert_eq!(SlabSize::S64.slots_per_page(), 64);
    assert_eq!(SlabSize::S512.slots_per_page(), 8);
}

// --- Slab (page) ---

#[test]
fn slab_full_init_and_alloc() {
    use crate::xgc::slab::page::Slab;
    let mut s = Slab::new(SlabSize::S32, 0x10000);
    s.init_full();
    assert_eq!(s.free_count(), 128);
    let slot = s.alloc().unwrap();
    assert!(slot < 128);
    assert_eq!(s.free_count(), 127);
}

#[test]
fn slab_alloc_exhaustion() {
    use crate::xgc::slab::page::Slab;
    let mut s = Slab::new(SlabSize::S16, 0x10000);
    s.init_full();
    let total = s.total();
    for _ in 0..total {
        assert!(s.alloc().is_some());
    }
    assert!(s.alloc().is_none());
    assert!(!s.has_free());
}

#[test]
fn slab_free_recycles() {
    use crate::xgc::slab::page::Slab;
    let mut s = Slab::new(SlabSize::S64, 0x20000);
    s.init_full();
    let slot = s.alloc().unwrap();
    s.free(slot);
    assert!(s.has_free());
    let again = s.alloc().unwrap();
    assert_eq!(slot, again);
}

// --- SlabAllocator ---

#[test]
fn slab_alloc_grows_first_time() {
    let mut a = SlabAllocator::new(0x1_0000_0000);
    assert_eq!(a.slab_count(), 0);
    let p = a.alloc(16).unwrap();
    assert_eq!(a.slab_count(), 1);
    assert_eq!(a.reserved(), 4096);
    assert!(p.addr() >= 0x1_0000_0000);
}

#[test]
fn slab_alloc_serves_from_partial() {
    let mut a = SlabAllocator::new(0x1000);
    let p1 = a.alloc(8).unwrap();
    let p2 = a.alloc(8).unwrap();
    assert_ne!(p1.addr(), p2.addr(), "different slots");
    assert_eq!(a.slab_count(), 1, "served from same slab");
}

#[test]
fn slab_free_keeps_slab() {
    let mut a = SlabAllocator::new(0x1000);
    let p = a.alloc(8).unwrap();
    a.free(p.addr() as u64, 8);
    // Slab stays allocated, but a future alloc may reuse it.
    let p2 = a.alloc(8).unwrap();
    assert_eq!(p.addr(), p2.addr());
}

// --- WorkDeque ---

#[test]
fn deque_lifo_local() {
    let mut d = WorkDeque::new();
    d.push(ColoredPtr::new(0x10, Color::White));
    d.push(ColoredPtr::new(0x20, Color::White));
    d.push(ColoredPtr::new(0x30, Color::White));
    assert_eq!(d.pop().unwrap().addr(), 0x30, "LIFO: 30 first");
    assert_eq!(d.pop().unwrap().addr(), 0x20);
    assert_eq!(d.pop().unwrap().addr(), 0x10);
    assert!(d.pop().is_none());
}

#[test]
fn deque_fifo_steal() {
    let mut d = WorkDeque::new();
    d.push(ColoredPtr::new(0x10, Color::White));
    d.push(ColoredPtr::new(0x20, Color::White));
    d.push(ColoredPtr::new(0x30, Color::White));
    assert_eq!(d.steal().unwrap().addr(), 0x10, "FIFO: 10 first");
    assert_eq!(d.steal().unwrap().addr(), 0x20);
    assert_eq!(d.pop().unwrap().addr(), 0x30, "still LIFO locally");
}

#[test]
fn deque_pop_on_empty() {
    let mut d = WorkDeque::new();
    assert!(d.pop().is_none());
    assert!(d.steal().is_none());
    assert!(d.is_empty());
}

#[test]
fn deque_capacity_limit() {
    let mut d = WorkDeque::new();
    for i in 0..d.capacity() {
        assert!(d.push(ColoredPtr::new(i, Color::White)));
    }
    assert!(!d.push(ColoredPtr::new(999, Color::White)));
}

// --- WorkerPool ---

#[test]
fn pool_register() {
    let mut p = WorkerPool::new();
    assert_eq!(p.worker_count(), 0);
    assert_eq!(p.register(), Some(0));
    assert_eq!(p.register(), Some(1));
    assert_eq!(p.worker_count(), 2);
}

#[test]
fn pool_local_push_and_steal() {
    let mut p = WorkerPool::new();
    let w0 = p.register().unwrap();
    let _w1 = p.register().unwrap();
    p.local_push(w0, ColoredPtr::new(0x100, Color::White));
    p.local_push(w0, ColoredPtr::new(0x200, Color::White));
    assert_eq!(p.total_pending(), 2);
    let stolen = p.steal(w1_id()).unwrap();
    assert_eq!(stolen.addr(), 0x100, "FIFO from victim's bottom");
}

#[test]
fn pool_local_pop_takes_lifo() {
    let mut p = WorkerPool::new();
    let w0 = p.register().unwrap();
    p.local_push(w0, ColoredPtr::new(0x100, Color::White));
    p.local_push(w0, ColoredPtr::new(0x200, Color::White));
    let top = p.local_pop(w0).unwrap();
    assert_eq!(top.addr(), 0x200);
}

#[test]
fn pool_steal_empty_returns_none() {
    let mut p = WorkerPool::new();
    let _w0 = p.register().unwrap();
    let w1 = p.register().unwrap();
    assert!(p.steal(w1).is_none());
}

fn w1_id() -> u8 {
    1
}
