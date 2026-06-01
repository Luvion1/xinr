//! Timed join tests.

use crate::sync::oneshot::Oneshot;
use crate::sync::timed_join::{TimedJoin, try_join_with_timeout};
use crate::sync::timer::TimerWheel;

#[test]
fn timed_join_already_ready() {
    let mut o: Oneshot<u32> = Oneshot::new();
    o.send(42).unwrap();
    let mut wheel = TimerWheel::new();
    let result = try_join_with_timeout(&mut o, &mut wheel, 100);
    match result.unwrap() {
        TimedJoin::Ready(v) => assert_eq!(v, 42),
        _ => panic!("expected Ready"),
    }
}

#[test]
fn timed_join_timeout() {
    let mut o: Oneshot<u32> = Oneshot::new();
    let mut wheel = TimerWheel::new();
    // No send — will timeout.
    let result = try_join_with_timeout(&mut o, &mut wheel, 100);
    match result.unwrap() {
        TimedJoin::Timeout => {}
        _ => panic!("expected Timeout"),
    }
}

#[test]
fn timed_join_drive_wheel() {
    let mut o: Oneshot<&'static str> = Oneshot::new();
    let mut wheel = TimerWheel::new();
    wheel.schedule(50, 0xAA);
    // First call should timeout (no value yet).
    let r1 = try_join_with_timeout(&mut o, &mut wheel, 50);
    assert!(matches!(r1, Ok(TimedJoin::Timeout)));
    // Now send a value.
    o.send("hello").unwrap();
    let r2 = try_join_with_timeout(&mut o, &mut wheel, 100);
    assert!(matches!(r2, Ok(TimedJoin::Ready("hello"))));
}
