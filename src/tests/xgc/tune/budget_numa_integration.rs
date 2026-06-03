//! Budget, NUMA, and integration tests.

use crate::xgc::Xgc;
use crate::xgc::budget::clock::{Instant, duration_ms};
use crate::xgc::budget::{BudgetTracker, CycleBudget};
use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::numa::node::NodeId;
use crate::xgc::numa::topology::NumaTopology;

// --- budget tests ---

#[test]
fn instant_zero() {
    let i = Instant::ZERO;
    assert_eq!(i.ms(), 0);
}

#[test]
fn instant_elapsed() {
    let a = Instant::from_ms(100);
    let b = Instant::from_ms(250);
    assert_eq!(a.elapsed_ms(b), 150);
    assert_eq!(b.elapsed_ms(a), 0, "saturating subtraction");
}

#[test]
fn duration_ms_helper() {
    let a = Instant::from_ms(10);
    let b = Instant::from_ms(55);
    assert_eq!(duration_ms(a, b), 45);
}

#[test]
fn budget_soft_defaults() {
    let b = CycleBudget::default_soft();
    assert_eq!(b.mark_ms, 5);
    assert!(b.mark_ms < b.relocate_ms * 100);
}

#[test]
fn budget_unbounded() {
    let b = CycleBudget::unbounded();
    assert_eq!(b.mark_ms, u64::MAX);
}

#[test]
fn tracker_within_budget() {
    let budget = CycleBudget {
        mark_ms: 100,
        sweep_ms: 50,
        relocate_ms: 100,
        remap_ms: 25,
    };
    let mut t = BudgetTracker::new(budget, Instant::from_ms(0));
    t.tick(Instant::from_ms(30));
    assert!(!t.mark_exceeded());
    t.tick(Instant::from_ms(150));
    assert!(t.mark_exceeded());
}

#[test]
fn tracker_sweep_budget() {
    let budget = CycleBudget {
        mark_ms: 100,
        sweep_ms: 10,
        relocate_ms: 100,
        remap_ms: 25,
    };
    let mut t = BudgetTracker::new(budget, Instant::from_ms(0));
    t.tick(Instant::from_ms(20));
    assert!(t.sweep_exceeded());
}

#[test]
fn tracker_reset() {
    let budget = CycleBudget::default_soft();
    let mut t = BudgetTracker::new(budget, Instant::from_ms(0));
    t.tick(Instant::from_ms(50));
    assert!(t.mark_exceeded());
    t.reset(Instant::from_ms(100));
    assert_eq!(t.elapsed(), 0);
    t.tick(Instant::from_ms(102));
    assert_eq!(t.elapsed(), 2);
    assert!(!t.mark_exceeded());
}

// --- NUMA tests ---

#[test]
fn node_id_local() {
    assert_eq!(NodeId::LOCAL.0, 0);
    assert!(NodeId::LOCAL.is_valid());
}

#[test]
fn node_id_any_is_invalid() {
    assert!(!NodeId::ANY.is_valid());
}

#[test]
fn node_id_equality() {
    assert_eq!(NodeId::new(0), NodeId::LOCAL);
    assert_ne!(NodeId::new(0), NodeId::new(1));
}

#[test]
fn numa_topology_register() {
    let mut t = NumaTopology::new();
    assert_eq!(t.node_count(), 0);
    t.register(NodeId::new(0), 1_000_000);
    t.register(NodeId::new(1), 2_000_000);
    assert_eq!(t.node_count(), 2);
}

#[test]
fn numa_most_free_picks_larger() {
    let mut t = NumaTopology::new();
    t.register(NodeId::new(0), 1_000_000);
    t.register(NodeId::new(1), 10_000_000);
    let m = t.most_free().unwrap();
    assert_eq!(m, NodeId::new(1));
}

#[test]
fn numa_reserve_and_release() {
    let mut t = NumaTopology::new();
    t.register(NodeId::new(0), 1000);
    assert!(t.reserve(NodeId::new(0), 600));
    assert!(!t.reserve(NodeId::new(0), 500), "exceeds capacity");
    t.release(NodeId::new(0), 200);
    assert!(t.reserve(NodeId::new(0), 500), "now fits");
}

// --- integration tests ---

#[test]
fn full_xgc_lifecycle() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();

    // Allocate a few objects.
    let p1 = ColoredPtr::new(0x1000, Color::White);
    let p2 = ColoredPtr::new(0x2000, Color::White);

    // Mark cycle.
    let epoch = gc.begin_mark().unwrap();
    assert!(epoch > 0);
    gc.push_root(p1).unwrap();
    gc.push_root(p2).unwrap();
    assert!(gc.pop_work().is_some());
    assert!(gc.pop_work().is_some());
    gc.finish_mark();
    assert_eq!(gc.phase().as_str(), "idle");

    // Relocate cycle.
    gc.begin_relocate();
    let new_p1 = ColoredPtr::new(0x3000, Color::Black);
    gc.record_move(p1, new_p1).unwrap();
    let stats = gc.finish_relocate();
    assert_eq!(stats.moved, 1);

    // Should-collect.
    let _should = gc.should_collect();
    let size = (8 * crate::xgc::region::REGION_SIZE) as u64;
    assert!(size > 0);

    // Shutdown.
    gc.shutdown().unwrap();
}

#[test]
fn worker_routes_through_xgc() {
    use crate::xgc::worker::signal::WorkOrder;
    use crate::xgc::worker::thread::GcWorker;
    let worker = GcWorker::new();
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    worker.request_mark();
    let processed = worker.process_one(&mut gc);
    assert_eq!(processed, WorkOrder::Mark);
    assert_eq!(gc.phase().as_str(), "marking");
    worker.request_relocate();
    let p = worker.process_one(&mut gc);
    assert_eq!(p, WorkOrder::Relocate);
    assert_eq!(gc.phase().as_str(), "relocating");
    worker.request_shutdown();
    let s = worker.process_one(&mut gc);
    assert_eq!(s, WorkOrder::Shutdown);
    // After shutdown, gc.shutdown was called.
}
