//! SATB barrier tests.

use crate::xgc::barrier::satb::SatbBuffer;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn satb_records_and_drains() {
    let mut buf = SatbBuffer::new();
    assert!(buf.is_empty());
    buf.record(ColoredPtr::new(0x1000, Color::White)).unwrap();
    buf.record(ColoredPtr::new(0x2000, Color::Black)).unwrap();
    assert!(!buf.is_empty());
    let mut count = 0;
    buf.drain(|_| count += 1);
    assert_eq!(count, 2);
}

#[test]
fn satb_errors_when_full() {
    let mut buf = SatbBuffer::new();
    let mut succeeded = 0;
    for _ in 0..257 {
        if buf.record(ColoredPtr::new(0x1000, Color::White)).is_ok() {
            succeeded += 1;
        }
    }
    assert_eq!(succeeded, 256);
    buf.drain(|_| {});
    assert!(buf.is_empty());
}
