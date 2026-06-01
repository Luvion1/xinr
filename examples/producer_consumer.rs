//! Example: producer-consumer with parking-based blocking send/recv.
//!
//! Demonstrates a producer thread that parks when the channel is full,
//! and a consumer that parks when empty. A timer wheel drives the
//! parking expiry.
//!
//! Run with: `cargo run --features alloc --example producer_consumer`

#![cfg(feature = "alloc")]

use xinr::sync::blocking::BlockingChannel;
use xinr::sync::blocking::RecvOutcome;
use xinr::sync::metrics::Metrics;
use xinr::sync::parking::lot::ParkingLot;
use xinr::sync::timer::TimerWheel;
use xinr::sync::waker::Waker;

fn main() {
    println!("=== Producer-consumer demo ===\n");

    // Channel of capacity 2: producer parks if full, consumer parks if empty.
    let mut ch: BlockingChannel<u32, 2> = BlockingChannel::new();

    // Parking lot for waiting threads.
    let mut lot = ParkingLot::new();

    // Timer wheel for periodic wakeups.
    let mut wheel = TimerWheel::new();
    let mut waker = Waker::new();

    // Metrics.
    let metrics = Metrics::new();

    // ---- Producer phase ----
    println!("[producer] sending 5 items into capacity-2 channel...");
    let producer_thread = 1u64;
    let mut sent = 0;
    for v in 10..=50u32 {
        match ch.send(v, producer_thread, v as u64 * 10) {
            Ok(Some(permit)) => {
                // Parked: schedule a wakeup at v*20.
                let parking_tok = permit.0;
                let timer_tok = v as u64 * 100;
                wheel.schedule(v as u64 * 20, timer_tok);
                waker.register(timer_tok, parking_tok);
                println!("  [producer] sent {} (parked, timer {})", v, timer_tok);
            }
            Ok(None) => {
                println!("  [producer] sent {}", v);
                sent += 1;
            }
            Err(_) => {
                break;
            }
        }
        metrics.inc_alloc();
    }
    println!("[producer] total sent without parking: {}\n", sent);

    // ---- Consumer phase ----
    println!("[consumer] draining...");
    let consumer_thread = 2u64;
    let mut total = 0u32;
    for i in 0..5 {
        match ch.recv(consumer_thread, 1000 + i as u64) {
            Ok(RecvOutcome::Value(v)) => {
                println!("  [consumer] recv {}", v);
                total += v;
            }
            Ok(RecvOutcome::Parked(p)) => {
                println!("  [consumer] would park with permit {:#x}", p.0);
            }
            Err(_) => {
                break;
            }
        }
        metrics.inc_free();
    }
    println!("[consumer] total received: {}\n", total);

    // ---- Drive timer wheel ----
    println!("[timer] advancing to t=200...");
    let woke = waker.drive(&mut wheel, &mut lot, 200);
    println!("[timer] woke {} threads\n", woke);

    // ---- Snapshot metrics ----
    let snap = metrics.snapshot();
    println!(
        "[metrics] allocs={}, frees={}, marks={}, cycles={}",
        snap[0], snap[1], snap[2], snap[6]
    );
    println!("[parking] still parked: {}", lot.parked_count());

    ch.close();
    println!("\n=== Producer-consumer demo complete ===");
}
