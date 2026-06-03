//! Colored pointer tests.

use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn colored_ptr_stores_address_and_color() {
    let p = ColoredPtr::new(0x1000, Color::Black);
    assert_eq!(p.addr(), 0x1000);
    assert_eq!(p.color(), Color::Black);
}

#[test]
fn colored_ptr_eq_matches_both() {
    let a = ColoredPtr::new(0x1000, Color::White);
    let b = ColoredPtr::new(0x1000, Color::White);
    let c = ColoredPtr::new(0x1000, Color::Black);
    assert_eq!(a, b);
    assert_ne!(a, c);
}
