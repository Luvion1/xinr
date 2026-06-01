//! Condvar tests.

use crate::sync::condvar::Condvar;

#[test]
fn condvar_new() {
    let cv = Condvar::new("test");
    assert_eq!(cv.name(), "test");
    assert_eq!(cv.waiter_count(), 0);
}

#[test]
fn condvar_register_and_notify() {
    let mut cv = Condvar::new("cv1");
    let permit = cv.acquire_permit();
    let token = cv.register_waiter(42, permit, 100).unwrap();
    assert_eq!(cv.waiter_count(), 1);
    let rec = cv.notify_one(token).unwrap();
    assert_eq!(rec.thread_id, 42);
    assert_eq!(cv.waiter_count(), 0);
}

#[test]
fn condvar_notify_by_thread() {
    let mut cv = Condvar::new("cv2");
    let permit = cv.acquire_permit();
    cv.register_waiter(99, permit, 50).unwrap();
    let rec = cv.notify_one_thread(99).unwrap();
    assert_eq!(rec.thread_id, 99);
}

#[test]
fn condvar_notify_all() {
    let mut cv = Condvar::new("cv3");
    for tid in 1..=5u64 {
        let p = cv.acquire_permit();
        cv.register_waiter(tid, p, 0).unwrap();
    }
    assert_eq!(cv.waiter_count(), 5);
    let woken = cv.notify_all();
    assert_eq!(woken, 5);
    assert_eq!(cv.waiter_count(), 0);
}
