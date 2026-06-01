//! Notify, WaitGroup, and select! macro tests.

extern crate alloc;
use alloc::vec::Vec;

use crate::RuntimeError;
use crate::sync::channel::BoundedChannel;
use crate::sync::notify::Notify;
use crate::sync::select::{SelectResult, select_recv_4};
use crate::sync::waitgroup::WaitGroup;

// --- Notify ---

#[test]
fn notify_new_is_unset() {
    let n = Notify::new();
    assert!(!n.is_notified());
    assert_eq!(n.waiters(), 0);
}

#[test]
fn notify_set_and_check() {
    let mut n = Notify::new();
    n.notify_one();
    assert!(n.is_notified());
}

#[test]
fn notify_wait_when_set() {
    let mut n = Notify::new();
    n.notify_one();
    assert!(n.wait().is_ok());
    assert_eq!(n.waiters(), 1);
}

#[test]
fn notify_wait_when_unset() {
    let mut n = Notify::new();
    assert!(matches!(n.wait(), Err(RuntimeError::WouldBlock)));
    assert_eq!(n.waiters(), 1);
    n.leave();
    assert_eq!(n.waiters(), 0);
}

#[test]
fn notify_clear() {
    let mut n = Notify::new();
    n.notify_one();
    n.clear();
    assert!(!n.is_notified());
}

#[test]
fn notify_all() {
    let mut n = Notify::new();
    n.notify_all();
    assert!(n.is_notified());
}

// --- WaitGroup ---

#[test]
fn waitgroup_new_zero() {
    let wg = WaitGroup::new();
    assert_eq!(wg.count(), 0);
    assert_eq!(wg.waiting(), 0);
}

#[test]
fn waitgroup_add_and_done() {
    let mut wg = WaitGroup::new();
    wg.add(3);
    assert_eq!(wg.count(), 3);
    assert!(matches!(wg.done(), Err(RuntimeError::WouldBlock)));
    assert!(matches!(wg.done(), Err(RuntimeError::WouldBlock)));
    assert!(wg.done().is_ok(), "third done reaches zero");
    assert_eq!(wg.count(), 0);
}

#[test]
fn waitgroup_wait_zero() {
    let mut wg = WaitGroup::new();
    assert!(wg.wait().is_ok());
}

#[test]
fn waitgroup_wait_nonzero() {
    let mut wg = WaitGroup::new();
    wg.add(1);
    assert!(matches!(wg.wait(), Err(RuntimeError::WouldBlock)));
    assert_eq!(wg.waiting(), 1);
    wg.done().unwrap();
    wg.leave();
    assert_eq!(wg.waiting(), 0);
}

#[test]
fn waitgroup_done_underflow() {
    let mut wg = WaitGroup::new();
    assert!(matches!(wg.done(), Err(RuntimeError::WouldBlock)));
}

// --- select! integration with channels ---

#[test]
fn select_recv_priority_order() {
    let mut a: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 2> = BoundedChannel::new();
    d.try_send(4).unwrap();
    a.try_send(1).unwrap();
    b.try_send(2).unwrap();
    c.try_send(3).unwrap();
    let r = select_recv_4([&mut a, &mut b, &mut c, &mut d])
        .unwrap()
        .unwrap();
    assert_eq!(r.index, 0, "first non-empty channel wins");
    assert_eq!(r.value, 1);
    let consumed: Vec<SelectResult<u32>> = (0..3)
        .map(|_| {
            select_recv_4([&mut a, &mut b, &mut c, &mut d])
                .unwrap()
                .unwrap()
        })
        .collect();
    let total: u32 = consumed.iter().map(|r| r.value).sum();
    assert_eq!(total, 2 + 3 + 4);
}
