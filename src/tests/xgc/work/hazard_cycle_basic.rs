//! Hazard pointer and cycle detector tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::cycle::CycleDetector;
use crate::xgc::cycle::stats::{CycleCandidate, CycleStats};
use crate::xgc::finalize::weak::WeakTable;
use crate::xgc::hazard::record::HazardRecord;
use crate::xgc::hazard::slot::HazardSlot;

// --- hazard tests ---

#[test]
fn slot_starts_inactive() {
    let s = HazardSlot::new();
    assert!(!s.is_active());
    assert!(s.get().is_null());
}

#[test]
fn slot_publish_and_clear() {
    let mut s = HazardSlot::new();
    let mut x: u32 = 42;
    s.publish(&mut x as *mut u32);
    assert!(s.is_active());
    assert_eq!(s.get() as *mut u32, &mut x as *mut u32);
    s.clear();
    assert!(!s.is_active());
}

#[test]
fn record_publish_clear() {
    let mut r = HazardRecord::new();
    assert!(!r.any_active());
    let mut x = 0u8;
    r.publish(0, &mut x as *mut u8);
    assert!(r.is_active(0));
    assert!(!r.is_active(1));
    assert!(r.any_active());
    r.clear(0);
    assert!(!r.is_active(0));
    assert!(!r.any_active());
}

#[test]
fn record_clear_all() {
    let mut r = HazardRecord::new();
    let mut x = 0u8;
    let mut y = 0u8;
    r.publish(0, &mut x as *mut u8);
    r.publish(1, &mut y as *mut u8);
    assert!(r.any_active());
    r.clear_all();
    assert!(!r.any_active());
}

#[test]
fn record_oob_safe() {
    let mut r = HazardRecord::new();
    let mut x = 0u8;
    r.publish(99, &mut x as *mut u8);
    assert!(!r.any_active());
    r.clear(99);
}

// --- cycle tests ---

#[test]
fn cycle_stats_starts_zero() {
    let s = CycleStats::new();
    assert_eq!(s.candidates, 0);
    assert_eq!(s.reclaimed, 0);
    assert_eq!(s.hit_rate(), 0.0);
}

#[test]
fn cycle_stats_hit_rate() {
    let mut s = CycleStats::new();
    s.candidates = 4;
    s.reclaimed = 3;
    assert!((s.hit_rate() - 0.75).abs() < 1e-6);
}

#[test]
fn cycle_queue_basic() {
    let mut q = crate::xgc::cycle::queue::CycleQueue::new();
    assert!(q.is_empty());
    let c = CycleCandidate {
        object: ColoredPtr::new(0x1000, Color::Black),
        weak_ref: 7,
    };
    q.enqueue(c).unwrap();
    assert_eq!(q.len(), 1);
    let c2 = q.dequeue().unwrap();
    assert_eq!(c2.weak_ref, 7);
    assert!(q.is_empty());
}

#[test]
fn cycle_detector_submit_and_run() {
    let mut d = CycleDetector::new();
    let mut wt = WeakTable::new();
    let p = ColoredPtr::new(0x2000, Color::Black);
    let w = wt.create(p).expect("create weak");
    let c = CycleCandidate {
        object: p,
        weak_ref: w.0,
    };
    d.submit(c);
    assert_eq!(d.pending(), 1);
    let stats = d.run_pass(&wt);
    // Object is still alive → not reclaimed.
    assert_eq!(stats.reclaimed, 0);
    assert_eq!(d.pending(), 0);
}

#[test]
fn cycle_detector_reclaims_dead() {
    let mut d = CycleDetector::new();
    let mut wt = WeakTable::new();
    let p = ColoredPtr::new(0x3000, Color::Black);
    let w = wt.create(p).expect("create");
    wt.invalidate(p);
    let c = CycleCandidate {
        object: p,
        weak_ref: w.0,
    };
    d.submit(c);
    let stats = d.run_pass(&wt);
    assert_eq!(stats.reclaimed, 1);
}

#[test]
fn cycle_detector_dfs_threshold() {
    let mut d = CycleDetector::new();
    assert!(!d.should_dfs());
    d.dfs_threshold = 3;
    for _ in 0..3 {
        d.submit(CycleCandidate {
            object: ColoredPtr::new(0x100, Color::Black),
            weak_ref: 0,
        });
    }
    assert!(d.should_dfs());
}
