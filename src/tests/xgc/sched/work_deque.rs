//! Work deque tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::sched::WorkDeque;

#[test]
fn deque_push_pop_lifo() {
    let mut dq = WorkDeque::new();
    dq.push(ColoredPtr::new(0x1000, Color::White));
    dq.push(ColoredPtr::new(0x2000, Color::Black));
    let p1 = dq.pop().unwrap();
    let p2 = dq.pop().unwrap();
    assert_eq!(p1.addr(), 0x2000);
    assert_eq!(p2.addr(), 0x1000);
}

#[test]
fn deque_steal_fifos() {
    let mut dq = WorkDeque::new();
    dq.push(ColoredPtr::new(0x1000, Color::White));
    dq.push(ColoredPtr::new(0x2000, Color::Black));
    let s1 = dq.steal().unwrap();
    let s2 = dq.steal().unwrap();
    assert_eq!(s1.addr(), 0x1000);
    assert_eq!(s2.addr(), 0x2000);
}