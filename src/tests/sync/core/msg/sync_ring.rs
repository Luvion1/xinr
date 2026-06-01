//! Bounded ring buffer tests.

use crate::RuntimeError;
use crate::sync::channel::ring::BoundedRing;

#[test]
fn ring_basic_push_pop() {
    let mut r: BoundedRing<u32, 4> = BoundedRing::new();
    assert_eq!(r.capacity(), 4);
    assert!(r.is_empty());
    assert!(!r.is_full());
    r.try_push(10).unwrap();
    r.try_push(20).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(r.try_pop().unwrap(), 10);
    assert_eq!(r.try_pop().unwrap(), 20);
    assert!(r.is_empty());
}

#[test]
fn ring_full_rejects() {
    let mut r: BoundedRing<u8, 2> = BoundedRing::new();
    r.try_push(1).unwrap();
    r.try_push(2).unwrap();
    assert!(r.is_full());
    assert_eq!(r.try_push(3), Err(RuntimeError::WouldBlock));
}

#[test]
fn ring_empty_rejects() {
    let mut r: BoundedRing<u8, 2> = BoundedRing::new();
    assert_eq!(r.try_pop(), Err(RuntimeError::WouldBlock));
}

#[test]
fn ring_wraps() {
    let mut r: BoundedRing<u8, 3> = BoundedRing::new();
    r.try_push(0).unwrap();
    r.try_push(1).unwrap();
    r.try_push(2).unwrap();
    assert_eq!(r.try_pop().unwrap(), 0);
    assert_eq!(r.try_pop().unwrap(), 1);
    r.try_push(3).unwrap();
    r.try_push(4).unwrap();
    assert_eq!(r.try_pop().unwrap(), 2);
    assert_eq!(r.try_pop().unwrap(), 3);
    assert_eq!(r.try_pop().unwrap(), 4);
    assert!(r.is_empty());
}

#[test]
fn ring_close() {
    let mut r: BoundedRing<u8, 2> = BoundedRing::new();
    r.close();
    assert_eq!(r.try_push(1), Err(RuntimeError::Closed));
    assert_eq!(r.try_pop(), Err(RuntimeError::Closed));
    r.close();
    assert!(r.is_closed());
}
