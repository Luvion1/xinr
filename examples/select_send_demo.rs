//! Example: select_send_4 / select_send_8 demo.
//!
//! Run with: `cargo run --features alloc --example select_send_demo`

use xinr::sync::channel::BoundedChannel;
use xinr::sync::select::{select_recv_4, select_send_4};

fn main() {
    println!("=== select_send demo ===\n");

    let mut a: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 4> = BoundedChannel::new();

    // Fill all but `c` to capacity.
    for _ in 0..4 {
        a.try_send(1).unwrap();
        b.try_send(2).unwrap();
        d.try_send(3).unwrap();
    }

    // Now select_send_4 should pick `c` as the only available.
    let r = select_send_4([&mut a, &mut b, &mut c, &mut d], 99);
    println!("send result: accepted={:?}", r.accepted);
    assert_eq!(r.accepted, Some(2));

    // Select recv picks the first ready: `a`.
    let channels = [&mut a, &mut b, &mut c, &mut d];
    let r = select_recv_4(channels).unwrap().unwrap();
    println!("recv from channel {} value={}", r.index, r.value);
    assert_eq!(r.index, 0);

    // Drain remaining values.
    let mut total = 1u32;
    for _ in 0..24 {
        let channels = [&mut a, &mut b, &mut c, &mut d];
        if let Some(r) = select_recv_4(channels).unwrap() {
            total += r.value;
        }
    }
    // 4*1 + 4*2 + 1*99 + 4*3 = 4 + 8 + 99 + 12 = 123
    assert_eq!(total, 123);
    println!("All channels drained, sum check passed.");
}
