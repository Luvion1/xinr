//! Select, SPSC, MPSC tests.

use crate::sync::channel::BoundedChannel;
use crate::sync::select::{select_recv_4, select_recv_8};
use crate::sync::spsc::{MpscChannel, SpscChannel};

// --- SPSC ---

#[test]
fn spsc_basic() {
    let mut ch: SpscChannel<u8, 4> = SpscChannel::new();
    assert!(ch.is_empty());
    ch.try_send(1).unwrap();
    ch.try_send(2).unwrap();
    assert_eq!(ch.try_recv().unwrap(), 1);
    assert_eq!(ch.try_recv().unwrap(), 2);
    assert!(ch.try_recv().is_err());
}

#[test]
fn spsc_full() {
    let mut ch: SpscChannel<u8, 2> = SpscChannel::new();
    ch.try_send(1).unwrap();
    ch.try_send(2).unwrap();
    assert!(ch.is_full());
    assert!(ch.try_send(3).is_err());
}

#[test]
fn spsc_wrap() {
    let mut ch: SpscChannel<u8, 3> = SpscChannel::new();
    ch.try_send(0).unwrap();
    ch.try_send(1).unwrap();
    ch.try_send(2).unwrap();
    assert_eq!(ch.try_recv().unwrap(), 0);
    ch.try_send(3).unwrap();
    assert_eq!(ch.try_recv().unwrap(), 1);
    assert_eq!(ch.try_recv().unwrap(), 2);
    assert_eq!(ch.try_recv().unwrap(), 3);
}

#[test]
fn spsc_capacity_and_len() {
    let ch: SpscChannel<u32, 8> = SpscChannel::new();
    assert_eq!(ch.capacity(), 8);
    assert_eq!(ch.len(), 0);
}

// --- MPSC ---

#[test]
fn mpsc_basic() {
    let mut ch: MpscChannel<u8, 4> = MpscChannel::new();
    assert!(ch.try_send(1).is_ok());
    assert!(ch.try_send(2).is_ok());
    assert_eq!(ch.try_recv().unwrap(), 1);
    assert_eq!(ch.try_recv().unwrap(), 2);
}

#[test]
fn mpsc_full() {
    let mut ch: MpscChannel<u8, 2> = MpscChannel::new();
    ch.try_send(1).unwrap();
    ch.try_send(2).unwrap();
    assert!(ch.is_full());
    assert!(ch.try_send(3).is_err());
}

#[test]
fn mpsc_capacity() {
    let ch: MpscChannel<u32, 16> = MpscChannel::new();
    assert_eq!(ch.capacity(), 16);
    assert_eq!(ch.len(), 0);
    assert!(ch.is_empty());
}

// --- Select ---

#[test]
fn select_recv_first() {
    let mut a: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 4> = BoundedChannel::new();
    b.try_send(100).unwrap();
    c.try_send(200).unwrap();
    let result = select_recv_4([&mut a, &mut b, &mut c, &mut d])
        .unwrap()
        .unwrap();
    assert_eq!(result.index, 1, "b is first non-empty");
    assert_eq!(result.value, 100);
}

#[test]
fn select_recv_empty() {
    let mut a: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 4> = BoundedChannel::new();
    let r = select_recv_4([&mut a, &mut b, &mut c, &mut d]).unwrap();
    assert!(r.is_none());
}

#[test]
fn select_recv_8_finds_fifth() {
    let mut c0: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c1: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c2: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c3: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c4: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c5: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c6: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c7: BoundedChannel<u32, 2> = BoundedChannel::new();
    c5.try_send(555).unwrap();
    let result = select_recv_8([
        &mut c0, &mut c1, &mut c2, &mut c3, &mut c4, &mut c5, &mut c6, &mut c7,
    ])
    .unwrap()
    .unwrap();
    assert_eq!(result.index, 5);
    assert_eq!(result.value, 555);
}
