//! Hazard slot tests.

use crate::xgc::hazard::slot::HazardSlot;

#[test]
fn slot_inactive_initially() {
    let s = HazardSlot::new();
    assert!(!s.is_active());
}

#[test]
fn slot_can_publish_and_clear() {
    let mut s = HazardSlot::new();
    s.publish(0x1000 as *mut u8);
    assert!(s.is_active());
    s.clear();
    assert!(!s.is_active());
}
