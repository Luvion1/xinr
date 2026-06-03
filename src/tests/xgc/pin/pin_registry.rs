//! Pin registry tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::pin::PinRegistry;

#[test]
fn registry_starts_empty() {
    let reg = PinRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn pin_creates_valid_entry() {
    let mut reg = PinRegistry::new();
    let ptr = ColoredPtr::new(0x1000, Color::White);
    let handle = reg.pin(ptr).unwrap();
    assert!(handle.is_valid());
    assert_eq!(reg.len(), 1);
}

#[test]
fn pin_twice_increments_count() {
    let mut reg = PinRegistry::new();
    let ptr = ColoredPtr::new(0x1000, Color::White);
    let h1 = reg.pin(ptr).unwrap();
    let h2 = reg.pin(ptr).unwrap();
    assert_eq!(reg.len(), 1, "same object, one entry");
    reg.unpin(h1);
    reg.unpin(h2);
    assert!(reg.is_empty());
}