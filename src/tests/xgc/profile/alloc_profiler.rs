//! Alloc profiler tests.

use crate::xgc::profile::AllocProfiler;
use crate::xgc::profile::site::SiteId;

#[test]
fn profiler_records_allocations() {
    let mut p = AllocProfiler::new();
    let id = SiteId::fresh();
    p.record_alloc(id, 1000);
    p.record_alloc(id, 500);
    let s = p.get(id).unwrap();
    assert_eq!(s.alloc_bytes, 1500);
}

#[test]
fn profiler_total_bytes() {
    let mut p = AllocProfiler::new();
    let a = SiteId::fresh();
    let b = SiteId::fresh();
    p.record_alloc(a, 1000);
    p.record_alloc(b, 2000);
    assert_eq!(p.total_alloc_bytes(), 3000);
}