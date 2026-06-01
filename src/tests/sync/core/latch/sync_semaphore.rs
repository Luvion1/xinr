//! Semaphore tests.

use crate::RuntimeError;
use crate::sync::semaphore::Semaphore;

#[test]
fn semaphore_initial_permits() {
    let s = Semaphore::new(5);
    assert_eq!(s.permits(), 5);
    assert_eq!(s.max(), 5);
}

#[test]
fn semaphore_acquire_release() {
    let mut s = Semaphore::new(2);
    s.try_acquire().unwrap();
    s.try_acquire().unwrap();
    assert_eq!(s.try_acquire(), Err(RuntimeError::WouldBlock));
    s.release().unwrap();
    s.try_acquire().unwrap();
}

#[test]
fn semaphore_overflow() {
    let mut s = Semaphore::new(2);
    assert_eq!(
        s.release(),
        Err(RuntimeError::Closed),
        "new(2) is already at max"
    );
    s.try_acquire().unwrap();
    s.release().unwrap();
    assert_eq!(
        s.release(),
        Err(RuntimeError::Closed),
        "back at max after release"
    );
}

#[test]
fn semaphore_waiting_tracking() {
    let mut s = Semaphore::new(1);
    s.record_wait();
    s.record_wait();
    assert_eq!(s.waiting(), 2);
    s.record_leave();
    assert_eq!(s.waiting(), 1);
}
