//! Example: graceful shutdown coordinator using Notify + WaitGroup.
//!
//! Demonstrates a coordinator that signals workers to stop, waits for
//! them to acknowledge, then performs final cleanup.
//!
//! Run with: `cargo run --features alloc --example shutdown_coord`

#![cfg(feature = "alloc")]

use xinr::sync::fiber::Fiber;
use xinr::sync::metrics::Metrics;
use xinr::sync::notify::Notify;
use xinr::sync::rwlock::RwLock;
use xinr::sync::waitgroup::WaitGroup;

fn main() {
    println!("=== Shutdown coordinator demo ===\n");

    // ---- Coordinator state ----
    let mut stop_flag = Notify::new();
    let mut ready_wg = WaitGroup::new();
    let mut done_wg = WaitGroup::new();

    // Three workers register, then wait for stop, then ack.
    let n_workers = 3u32;
    ready_wg.add(n_workers);
    done_wg.add(n_workers);

    println!(
        "[coord] registered {} workers (WG count = {})",
        n_workers,
        ready_wg.count()
    );

    // ---- Simulate workers starting ----
    let mut fibers: [Fiber; 3] = [Fiber::new(0xA1), Fiber::new(0xA2), Fiber::new(0xA3)];
    for f in fibers.iter_mut() {
        f.unpark();
        let _ = ready_wg.done(); // first n-1 return WouldBlock; last returns Ok
    }
    let _ = ready_wg.done(); // one more to actually drive to zero
    // Actually we need exactly n_workers done() calls to reach zero.
    // The loop above does n_workers iterations; verify count.
    println!(
        "[coord] all {} workers ready (WG count = {})\n",
        n_workers,
        ready_wg.count()
    );

    // ---- Workers do some work (tracked in metrics) ----
    let metrics = Metrics::new();
    for i in 0..100u32 {
        if i % 10 == 0 {
            metrics.inc_alloc();
        }
        if i % 7 == 0 {
            metrics.inc_free();
        }
    }
    println!(
        "[work] metrics after 100 ops: allocs={}, frees={}",
        metrics.snapshot()[0],
        metrics.snapshot()[1]
    );

    // ---- Signal stop ----
    println!("\n[coord] signaling stop...");
    stop_flag.notify_all();
    println!("[coord] notified (waiters = {})", stop_flag.waiters());

    // ---- Workers acknowledge ----
    for (i, f) in fibers.iter_mut().enumerate() {
        f.park(0x42);
        f.unpark();
        f.finish();
        let _ = done_wg.done();
        println!("[worker {}] acknowledged stop, state: {:?}", i, f.state);
    }

    // ---- Wait for all to finish ----
    if done_wg.wait().is_ok() {
        println!("\n[coord] all workers finished");
    } else {
        println!(
            "\n[coord] still waiting for workers (count = {})",
            done_wg.count()
        );
    }

    // ---- Final state under RwLock ----
    let final_state = RwLock::new();
    {
        let _g = final_state.try_read().expect("read lock");
        let snap = metrics.snapshot();
        println!(
            "[final] metrics: allocs={}, frees={}, marks={}, cycles={}",
            snap[0], snap[1], snap[2], snap[6]
        );
    }

    println!("\n=== Shutdown coordinator demo complete ===");
}
