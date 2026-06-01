//! Bounded channel tests.

use crate::RuntimeError;
use crate::sync::channel::BoundedChannel;

#[test]
fn channel_round_trip() {
    let mut ch: BoundedChannel<u32, 4> = BoundedChannel::new();
    ch.try_send(1).unwrap();
    ch.try_send(2).unwrap();
    ch.try_send(3).unwrap();
    assert_eq!(ch.try_recv().unwrap(), 1);
    assert_eq!(ch.try_recv().unwrap(), 2);
    assert_eq!(ch.try_recv().unwrap(), 3);
    assert_eq!(ch.try_recv(), Err(RuntimeError::WouldBlock));
}

#[test]
fn channel_mpmc_pattern() {
    let mut ch: BoundedChannel<u8, 2> = BoundedChannel::new();
    ch.try_send(0xAA).unwrap();
    ch.try_send(0xBB).unwrap();
    assert!(ch.is_full());
    assert_eq!(ch.try_recv().unwrap(), 0xAA);
    ch.try_send(0xCC).unwrap();
    let drained: u16 = ch.try_recv().unwrap() as u16 + ch.try_recv().unwrap() as u16;
    assert_eq!(drained, 0xBBu16 + 0xCCu16);
}
