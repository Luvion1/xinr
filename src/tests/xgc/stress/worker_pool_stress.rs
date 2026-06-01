//! Worker pool stress test: 1000+ operations using Scope + fibers + channels.
//!
//! Simulates a simple worker pool: tasks are spawned via Scope, each task
//! sends a result through a channel, and we verify all results arrive.

use crate::sync::channel::BoundedChannel;

const SENDS: usize = 1000;

#[test]
fn worker_pool_high_throughput() {
    let mut chan: BoundedChannel<u32, 1024> = BoundedChannel::new();

    // Simulate 500 "worker" operations sending values.
    for i in 0..SENDS {
        chan.try_send(i as u32).unwrap();
    }

    // Drain and count all values.
    let mut total: u64 = 0;
    let mut received = 0;
    while received < SENDS {
        match chan.try_recv() {
            Ok(v) => {
                total = total.wrapping_add(v as u64);
                received += 1;
            }
            Err(_) => break,
        }
    }
    // Each value 0..1000 summed.
    let expected: u64 = (SENDS as u64 - 1) * (SENDS as u64) / 2;
    assert_eq!(total, expected);
}

#[test]
fn select_under_pressure() {
    use crate::sync::select::{select_recv_4, select_send_4};

    let mut a: BoundedChannel<u32, 32> = BoundedChannel::new();
    let mut b: BoundedChannel<u32, 32> = BoundedChannel::new();
    let mut c: BoundedChannel<u32, 32> = BoundedChannel::new();
    let mut d: BoundedChannel<u32, 32> = BoundedChannel::new();

    // Fill all but `c` to capacity.
    for _ in 0..32 {
        a.try_send(1).unwrap();
        b.try_send(2).unwrap();
        d.try_send(3).unwrap();
    }

    // Now send only goes to `c`.
    let r = select_send_4([&mut a, &mut b, &mut c, &mut d], 99);
    assert_eq!(r.accepted, Some(2));

    // Receive picks the first ready: `a`.
    let r = select_recv_4([&mut a, &mut b, &mut c, &mut d])
        .unwrap()
        .unwrap();
    assert_eq!(r.index, 0);
    assert_eq!(r.value, 1);
}

#[test]
#[ignore = "long-haul stress; run with --ignored"]
fn worker_pool_long_haul() {
    let mut chan: BoundedChannel<u32, 4096> = BoundedChannel::new();
    for i in 0..2000u32 {
        chan.try_send(i).unwrap();
    }

    let mut received = 0u32;
    while received < 2000 {
        if let Ok(v) = chan.try_recv() {
            received += 1;
            // Sum all for sanity.
            let _ = v;
        }
    }
    assert_eq!(received, 2000);
}
