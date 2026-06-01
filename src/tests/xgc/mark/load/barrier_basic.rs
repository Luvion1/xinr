//! Barrier and SATB/ref-update buffer tests.

use crate::xgc::barrier::mark_state::MarkEpoch;
use crate::xgc::barrier::ref_update::RefUpdateBuffer;
use crate::xgc::barrier::satb::SatbBuffer;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn satb_record_and_drain() {
    let mut s = SatbBuffer::new();
    let p = ColoredPtr::new(0x1000, Color::White);
    s.record(p).unwrap();
    let mut seen = false;
    s.drain(|entry| {
        if entry == p {
            seen = true;
        }
    });
    assert!(seen);
    assert!(s.is_empty());
}

#[test]
fn ref_update_record_and_drain() {
    let mut r = RefUpdateBuffer::new();
    let p = ColoredPtr::new(0x2000, Color::Black);
    r.record(0xDEAD, p).unwrap();
    let mut seen_addr = 0;
    let mut seen_val: Option<ColoredPtr> = None;
    r.drain(|addr, val| {
        seen_addr = addr;
        seen_val = Some(val);
    });
    assert_eq!(seen_addr, 0xDEAD);
    assert_eq!(seen_val, Some(p));
}

#[test]
fn epoch_advances_monotonically() {
    let e = MarkEpoch::new();
    assert_eq!(e.current(), 0);
    let a = e.advance();
    let b = e.advance();
    assert!(b > a);
    assert!(e.is_stale(0));
    assert!(!e.is_stale(b));
}
