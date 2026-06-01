//! Metrics and cache-padded tests.

use crate::sync::cache_padded::{CACHE_LINE, CachePadded, PaddedCounter};
use crate::sync::metrics::Metrics;

#[test]
fn metrics_new_all_zero() {
    let m = Metrics::new();
    let s = m.snapshot();
    assert!(s.iter().all(|&v| v == 0));
}

#[test]
fn metrics_increments() {
    let m = Metrics::new();
    m.inc_alloc();
    m.inc_alloc();
    m.inc_free();
    m.inc_mark();
    m.inc_cycle();
    m.inc_cycle();
    m.inc_cycle();
    let s = m.snapshot();
    assert_eq!(s[0], 2, "allocs");
    assert_eq!(s[1], 1, "frees");
    assert_eq!(s[2], 1, "marks");
    assert_eq!(s[6], 3, "cycles");
}

#[test]
fn metrics_default() {
    let m = Metrics::default();
    assert_eq!(m.snapshot()[0], 0);
}

#[test]
fn padded_counter_basics() {
    let c = PaddedCounter::new();
    c.inc();
    c.inc();
    c.add(10);
    assert_eq!(c.load(), 12);
}

#[test]
fn padded_counter_default() {
    let c = PaddedCounter::default();
    assert_eq!(c.load(), 0);
}

#[test]
fn cache_line_size() {
    assert_eq!(CACHE_LINE, 64);
}

#[test]
fn cache_padded_wraps() {
    let p = CachePadded::new(42u32);
    assert_eq!(p.value, 42);
}
