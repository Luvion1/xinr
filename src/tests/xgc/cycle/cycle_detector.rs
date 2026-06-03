//! Cycle detector tests.

use crate::xgc::colored::ColoredPtr;
use crate::xgc::cycle::stats::CycleCandidate;
use crate::xgc::cycle::CycleDetector;

#[test]
fn detector_starts_empty() {
    let d = CycleDetector::new();
    assert_eq!(d.pending(), 0);
}

#[test]
fn detector_tracks_candidates() {
    let mut d = CycleDetector::new();
    d.submit(CycleCandidate { object: ColoredPtr::new(0x1000, crate::xgc::colored::Color::White), weak_ref: 1 });
    d.submit(CycleCandidate { object: ColoredPtr::new(0x2000, crate::xgc::colored::Color::White), weak_ref: 2 });
    assert_eq!(d.pending(), 2);
}

#[test]
fn dfs_threshold_triggers() {
    let mut d = CycleDetector::new();
    for i in 0..10usize {
        d.submit(CycleCandidate { object: ColoredPtr::new(i, crate::xgc::colored::Color::White), weak_ref: i as u32 });
    }
    assert!(d.should_dfs());
}