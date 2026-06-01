//! Blocking channel and integration tests.

use crate::RuntimeError;
use crate::sync::blocking::BlockingChannel;
use crate::sync::blocking::RecvOutcome;
use crate::sync::parking::permit::Permit;

// --- BlockingChannel ---

#[test]
fn blocking_send_succeeds_when_room() {
    let mut ch: BlockingChannel<u32, 4> = BlockingChannel::new();
    let outcome = ch.send(42, 1, 100).unwrap();
    assert!(outcome.is_none(), "did not park");
    assert_eq!(ch.len(), 1);
}

#[test]
fn blocking_send_parks_when_full() {
    let mut ch: BlockingChannel<u8, 1> = BlockingChannel::new();
    ch.try_send(1).unwrap();
    let outcome = ch.send(2, 7, 200).unwrap();
    let permit = outcome.expect("should park");
    assert!(permit.is_valid());
}

#[test]
fn blocking_recv_value() {
    let mut ch: BlockingChannel<u32, 2> = BlockingChannel::new();
    ch.try_send(99).unwrap();
    match ch.recv(1, 0).unwrap() {
        RecvOutcome::Value(v) => assert_eq!(v, 99),
        _ => panic!("expected Value"),
    }
}

#[test]
fn blocking_recv_parks_when_empty() {
    let mut ch: BlockingChannel<u8, 2> = BlockingChannel::new();
    match ch.recv(1, 0).unwrap() {
        RecvOutcome::Parked(p) => assert!(p.is_valid()),
        _ => panic!("expected Parked"),
    }
}

#[test]
fn blocking_wake_sender() {
    let mut ch: BlockingChannel<u8, 1> = BlockingChannel::new();
    ch.try_send(1).unwrap();
    let permit = ch.send(2, 7, 0).unwrap().unwrap();
    assert!(ch.wake_sender(permit.0));
    assert!(!ch.wake_sender(permit.0), "already woken");
}

#[test]
fn blocking_close() {
    let mut ch: BlockingChannel<u8, 2> = BlockingChannel::new();
    ch.close();
    assert!(ch.is_closed());
    assert_eq!(ch.try_send(1), Err(RuntimeError::Closed));
}

#[test]
fn blocking_recv_after_close() {
    let mut ch: BlockingChannel<u8, 2> = BlockingChannel::new();
    ch.close();
    assert_eq!(ch.recv(1, 0), Err(RuntimeError::Closed));
}

// --- Integration: scope + channel + fiber ---

#[test]
fn full_pipeline_scope_channel_fiber() {
    use crate::sync::channel::BoundedChannel;
    use crate::sync::fiber::Fiber;
    use crate::sync::scope::Scope;

    // Set up: 2 fibers, 1 channel, 1 scope.
    let mut ch: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut scope: Scope<2> = Scope::new();
    let f1 = Fiber::new(1);
    let f2 = Fiber::new(2);

    // Scope tracks both fibers as tasks.
    let _t1 = scope.try_spawn::<u32>().unwrap();
    let _t2 = scope.try_spawn::<u32>().unwrap();
    assert_eq!(scope.active(), 2);

    // Channel: producer side.
    for v in [10u32, 20, 30, 40] {
        ch.try_send(v).unwrap();
    }
    assert!(ch.is_full());
    let total: u32 = (0..4).map(|_| ch.try_recv().unwrap()).sum();
    assert_eq!(total, 100);

    // Both fibers eventually complete.
    scope.complete(0);
    scope.complete(1);
    assert!(scope.is_empty());
    scope.close().unwrap();

    // Both fibers in the stack pool are accounted for (by id).
    assert_eq!(f1.id.0, 1);
    assert_eq!(f2.id.0, 2);

    // The unused Permit sentinel is detectable.
    assert!(!Permit::NONE.is_valid());
}
