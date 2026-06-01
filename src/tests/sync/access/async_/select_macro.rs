//! `select!` macro tests.

use crate::select;
use crate::sync::channel::BoundedChannel;

#[test]
fn select_macro_picks_first_ready() {
    let mut a: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 4> = BoundedChannel::new();
    a.try_send(10).unwrap();
    b.try_send(20).unwrap();
    let result = select! {
        0 => recv(a) => 0,
        1 => recv(b) => 1,
    };
    assert_eq!(result, Some(0), "channel 0 is polled first");
}

#[test]
fn select_recv_4_finds_one() {
    let mut c0: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c1: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c2: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c3: BoundedChannel<u32, 4> = BoundedChannel::new();
    c2.try_send(42).unwrap();
    let r = crate::sync::select::select_recv_4([&mut c0, &mut c1, &mut c2, &mut c3]).unwrap();
    assert!(r.is_some());
    let sr = r.unwrap();
    assert_eq!(sr.index, 2);
    assert_eq!(sr.value, 42);
}
