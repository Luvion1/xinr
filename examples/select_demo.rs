//! Example: SPSC + MPSC + select! usage.
//!
//! Demonstrates single-producer/single-consumer and multi-producer/single-consumer
//! channels, plus the select helper for polling multiple receivers.
//!
//! Run with: `cargo run --features alloc --example select_demo`

#![cfg(feature = "alloc")]

use xinr::sync::channel::BoundedChannel;
use xinr::sync::select::{SelectResult, select_recv_4, select_send_4};
use xinr::sync::spsc::{MpscChannel, SpscChannel};

fn main() {
    // ---- 1. SPSC channel ----
    let mut spsc: SpscChannel<u8, 4> = SpscChannel::new();
    println!("SPSC channel capacity: {}", spsc.capacity());
    for v in 1..=4u8 {
        spsc.try_send(v).unwrap();
    }
    assert!(spsc.is_full());
    println!("SPSC filled, draining:");
    while let Ok(v) = spsc.try_recv() {
        print!("  {} ", v);
    }
    println!();

    // ---- 2. MPSC channel ----
    let mut mpsc: MpscChannel<&'static str, 4> = MpscChannel::new();
    println!("\nMPSC channel capacity: {}", mpsc.capacity());
    mpsc.try_send("a").unwrap();
    mpsc.try_send("b").unwrap();
    mpsc.try_send("c").unwrap();
    println!("MPSC: 3 producers sent, draining:");
    while let Ok(v) = mpsc.try_recv() {
        print!("  {} ", v);
    }
    println!();

    // ---- 3. select_recv_4: poll 4 channels ----
    let mut a: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 2> = BoundedChannel::new();
    c.try_send(300).unwrap();
    a.try_send(100).unwrap();

    println!("\nselect_recv_4 across [a=100, b=empty, c=300, d=empty]:");
    match select_recv_4([&mut a, &mut b, &mut c, &mut d]).unwrap() {
        Some(SelectResult { index, value }) => {
            println!("  -> first ready: ch[{}] = {}", index, value);
        }
        None => println!("  -> none ready"),
    }

    // Drain the rest.
    let mut next = select_recv_4([&mut a, &mut b, &mut c, &mut d]).unwrap();
    while let Some(SelectResult { index, value }) = next {
        println!("  -> next: ch[{}] = {}", index, value);
        next = select_recv_4([&mut a, &mut b, &mut c, &mut d]).unwrap();
    }
    println!("  -> all channels empty");

    // ---- 4. select_send_4 demo ----
    let mut x: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut y: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut z: BoundedChannel<u32, 2> = BoundedChannel::new();
    let mut w: BoundedChannel<u32, 2> = BoundedChannel::new();

    // Fill all but z.
    x.try_send(0).unwrap();
    y.try_send(0).unwrap();
    w.try_send(0).unwrap();

    let r = select_send_4([&mut x, &mut y, &mut z, &mut w], 42);
    println!(
        "\nselect_send_4: 3/4 full, send picks z: accepted={:?}",
        r.accepted
    );
    assert_eq!(r.accepted, Some(2));

    println!("\nSelect demo complete.");
}
