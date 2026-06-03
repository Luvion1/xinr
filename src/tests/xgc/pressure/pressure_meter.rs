//! Pressure meter tests.

use crate::xgc::pressure::PressureMeter;

#[test]
fn meter_starts_at_zero() {
    let m = PressureMeter::new();
    assert_eq!(m.live(), 0);
}

#[test]
fn meter_records_allocations() {
    let mut m = PressureMeter::new();
    m.record_alloc(1000);
    m.record_alloc(2000);
    assert_eq!(m.live(), 3000);
}