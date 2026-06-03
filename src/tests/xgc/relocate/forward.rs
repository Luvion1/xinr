//! Forward pointer tests for XGC.

use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn forward_returns_unchanged_when_relocate_inactive() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    let ptr = ColoredPtr::new(0x1000, Color::White);
    assert_eq!(gc.forward(ptr), ptr);
    gc.shutdown().unwrap();
}

#[test]
fn forward_returns_relocated_address() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    gc.begin_relocate();
    let old = ColoredPtr::new(0x1000, Color::White);
    let new = ColoredPtr::new(0x2000, Color::Black);
    gc.record_move(old, new).unwrap();
    let result = gc.forward(old);
    assert_eq!(result.addr(), 0x2000);
    gc.shutdown().unwrap();
}