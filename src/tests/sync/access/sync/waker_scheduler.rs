//! Waker and scheduler tests.

use crate::RuntimeError;
use crate::sync::parking::lot::ParkingLot;
use crate::sync::scheduler::Scheduler;
use crate::sync::timer::TimerWheel;
use crate::sync::waker::Waker;

// --- Waker ---

#[test]
fn waker_new() {
    let w = Waker::new();
    assert_eq!(w.count(), 0);
}

#[test]
fn waker_register_and_fire() {
    let mut w = Waker::new();
    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    let parking_tok = lot.park(99, permit, 0).unwrap();
    assert!(w.register(0xA, parking_tok));
    assert!(w.register(0xB, 0xDEAD)); // unknown parking token
    assert_eq!(w.count(), 2);

    let rec = w.fire(0xA, &mut lot).expect("fire A should succeed");
    assert_eq!(rec.thread_id, 99);
    assert!(w.fire(0xB, &mut lot).is_none(), "no thread for 0xB");
}

#[test]
fn waker_register_replaces() {
    let mut w = Waker::new();
    w.register(0xA, 0x1);
    w.register(0xA, 0x2); // replace
    assert_eq!(w.count(), 1);
}

#[test]
fn waker_cancel() {
    let mut w = Waker::new();
    w.register(0xA, 0x1);
    assert!(w.cancel(0xA));
    assert_eq!(w.count(), 0);
    assert!(!w.cancel(0xA));
}

#[test]
fn waker_full() {
    let mut w = Waker::new();
    for i in 0..64u64 {
        assert!(w.register(i, i));
    }
    assert!(!w.register(64, 64), "capacity full");
}

// --- Waker drive ---

#[test]
fn waker_drive_advances_wheel_and_unparks() {
    let mut w = Waker::new();
    let mut wheel = TimerWheel::new();
    let mut lot = ParkingLot::new();

    let permit = lot.acquire_permit();
    let parking_tok = lot.park(42, permit, 0).unwrap();
    wheel.schedule(100, 0xA);
    w.register(0xA, parking_tok);

    let woke = w.drive(&mut wheel, &mut lot, 100);
    assert_eq!(woke, 1, "thread 42 should be woken");
    assert_eq!(lot.parked_count(), 0);
}

// --- Scheduler ---

#[test]
fn scheduler_new() {
    let s = Scheduler::new();
    assert_eq!(s.count(), 0);
    assert_eq!(s.capacity(), 16);
}

#[test]
fn scheduler_register() {
    let mut s = Scheduler::new();
    assert_eq!(s.register().unwrap(), 0);
    assert_eq!(s.register().unwrap(), 1);
    assert_eq!(s.count(), 2);
}

#[test]
fn scheduler_run_next() {
    let mut s = Scheduler::new();
    let id = s.register().unwrap();
    let run = s.run_next().unwrap();
    assert_eq!(run, id);
}

#[test]
fn scheduler_run_empty() {
    let mut s = Scheduler::new();
    assert_eq!(s.run_next(), Err(RuntimeError::WouldBlock));
}

#[test]
fn scheduler_park_unpark() {
    let mut s = Scheduler::new();
    let id = s.register().unwrap();
    s.park_current(id, 0x42).unwrap();
    assert_eq!(
        s.state(id),
        Some(crate::sync::fiber::state::FiberState::Parked)
    );
    s.unpark(id).unwrap();
    assert_eq!(
        s.state(id),
        Some(crate::sync::fiber::state::FiberState::Running)
    );
}

#[test]
fn scheduler_finish() {
    let mut s = Scheduler::new();
    let id = s.register().unwrap();
    s.unpark(id).unwrap();
    s.finish(id).unwrap();
    assert!(s.state(id).unwrap().is_terminal());
}

#[test]
fn scheduler_round_robin() {
    let mut s = Scheduler::new();
    s.register().unwrap();
    s.register().unwrap();
    s.register().unwrap();
    // All start in Ready state. run_next should cycle through them.
    let mut seen = [false; 3];
    for _ in 0..3 {
        let id = s.run_next().unwrap();
        seen[id as usize] = true;
    }
    assert!(
        seen.iter().all(|&x| x),
        "all fibers should be picked exactly once"
    );
}
