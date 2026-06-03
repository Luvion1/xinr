//! Mark epoch tests.

use crate::xgc::barrier::mark_state::MarkEpoch;

#[test]
fn epoch_starts_at_zero() {
    let epoch = MarkEpoch::new();
    assert_eq!(epoch.current(), 0);
}

#[test]
fn epoch_advances_monotonically() {
    let epoch = MarkEpoch::new();
    let a = epoch.advance();
    let b = epoch.advance();
    assert!(b > a);
}

#[test]
fn epoch_detects_stale() {
    let epoch = MarkEpoch::new();
    let a = epoch.advance();
    assert!(!epoch.is_stale(a));
    epoch.advance();
    assert!(epoch.is_stale(a));
}