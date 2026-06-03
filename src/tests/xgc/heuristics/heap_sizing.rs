//! Heap sizing heuristics tests.

use crate::xgc::heuristics::sizing::HeapSizing;

#[test]
fn sizing_default_watermarks() {
    let s = HeapSizing::default_conservative();
    assert_eq!(s.high_watermark, 0.8);
    assert_eq!(s.low_watermark, 0.3);
}

#[test]
fn sizing_grow_calculation() {
    let s = HeapSizing::default_conservative();
    let grown = s.grown_capacity(1000);
    assert_eq!(grown, 1250);
}
