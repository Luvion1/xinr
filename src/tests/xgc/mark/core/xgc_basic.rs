//! End-to-end XGC lifecycle tests.

use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn new_xgc_succeeds() {
    let gc = Xgc::new(16);
    assert!(gc.is_ok());
}

#[test]
fn zero_regions_rejected() {
    let gc = Xgc::new(0);
    assert!(gc.is_err());
}

#[test]
fn init_then_shutdown() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().expect("first init");
    assert!(gc.init().is_err(), "double init must fail");
    gc.shutdown().expect("clean shutdown");
}

#[test]
fn shutdown_without_init_fails() {
    let mut gc = Xgc::new(8).unwrap();
    assert!(gc.shutdown().is_err());
}

#[test]
fn initial_phase_is_idle() {
    let gc = Xgc::new(8).unwrap();
    assert_eq!(gc.phase().as_str(), "idle");
}

#[test]
fn initial_epoch_is_zero() {
    let gc = Xgc::new(8).unwrap();
    assert_eq!(gc.epoch(), 0);
}

#[test]
fn mark_cycle_advances_epoch() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    let before = gc.epoch();
    let after = gc.begin_mark().expect("begin mark");
    assert!(after > before, "epoch must advance on mark");
    gc.finish_mark();
    assert_eq!(gc.phase().as_str(), "idle");
}

#[test]
fn colored_ptr_round_trip() {
    let p = ColoredPtr::new(0x1000, Color::Grey);
    assert_eq!(p.addr(), 0x1000);
    assert_eq!(p.color(), Color::Grey);
    assert!(!p.is_relocated());
}

#[test]
fn mark_then_relocate_coexists() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    gc.begin_mark().unwrap();
    let root = ColoredPtr::new(0x2000, Color::White);
    gc.push_root(root).expect("push root");
    assert!(gc.pop_work().is_some());
    gc.finish_mark();
    gc.begin_relocate();
    let old = ColoredPtr::new(0x2000, Color::Black);
    let new = ColoredPtr::new(0x3000, Color::Black);
    gc.record_move(old, new).expect("record");
    let stats = gc.finish_relocate();
    assert_eq!(stats.moved, 1);
    gc.shutdown().unwrap();
}
