//! GC statistics tests.

use crate::xgc::diagnostics::stats::GcStats;

#[test]
fn stats_default_is_zero() {
    let s = GcStats::default();
    assert_eq!(s.bytes_allocated, 0);
    assert_eq!(s.live_bytes(), 0);
}

#[test]
fn stats_records_allocation() {
    let mut s = GcStats::new();
    s.record_alloc(1000);
    s.record_alloc(2000);
    assert_eq!(s.bytes_allocated, 3000);
}

#[test]
fn stats_computes_live_bytes() {
    let mut s = GcStats::new();
    s.record_alloc(5000);
    s.record_free(1000);
    assert_eq!(s.live_bytes(), 4000);
}