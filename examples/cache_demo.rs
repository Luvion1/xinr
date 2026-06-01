//! Example: simple in-memory cache with metrics tracking.
//!
//! Demonstrates metrics + cache-padded counters + parking for an LRU-like
//! cache with hit/miss tracking.
//!
//! Run with: `cargo run --features alloc --example cache_demo`

#![cfg(feature = "alloc")]

use xinr::sync::cache_padded::PaddedCounter;
use xinr::sync::channel::BoundedChannel;
use xinr::sync::metrics::Metrics;
use xinr::sync::parking::lot::ParkingLot;
use xinr::sync::rwlock::RwLock;

const CACHE_SIZE: usize = 4;

/// One cache entry: key-value pair.
#[derive(Clone, Copy, Default)]
struct Entry {
    key: u32,
    value: u32,
    valid: bool,
}

impl Entry {
    fn matches(&self, key: u32) -> bool {
        self.valid && self.key == key
    }
}

fn main() {
    println!("=== Cache demo ===\n");

    // ---- 1. Metrics ----
    let metrics = Metrics::new();
    println!("[metrics] initialized (all counters = 0)");

    // ---- 2. Padded counters for per-thread stats ----
    let thread_a_hits = PaddedCounter::new();
    let thread_b_hits = PaddedCounter::new();

    // ---- 3. Cache protected by RwLock ----
    let mut cache = RwLock::new();
    let mut backing: [Entry; CACHE_SIZE] = [Entry::default(); CACHE_SIZE];

    // ---- 4. Bounded channel for cache eviction events ----
    let mut evict_ch: BoundedChannel<u32, 4> = BoundedChannel::new();

    // ---- 5. Park / unpark for misses ----
    let mut lot = ParkingLot::new();
    let mut misses: u32 = 0;

    // Simulate 10 lookups.
    for i in 0..10u32 {
        let key = i % 5; // keys 0..4, so we get repeats
        metrics.inc_alloc();

        // Try a read lock.
        let g = cache.try_read().expect("read lock");
        let mut found = false;
        for e in backing.iter() {
            if e.matches(key) {
                thread_a_hits.inc();
                found = true;
                break;
            }
        }
        g.release();

        if found {
            metrics.inc_free();
        } else {
            // Miss: acquire write lock, insert, log eviction if needed.
            let permit = lot.acquire_permit();
            let token = lot.park(1, permit, 0).unwrap();
            let _ = token;
            let wg = cache.try_write(1).expect("write lock");
            let mut found_free = false;
            for e in backing.iter_mut() {
                if !e.valid {
                    *e = Entry {
                        key,
                        value: key * 10,
                        valid: true,
                    };
                    misses += 1;
                    metrics.inc_eviction();
                    found_free = true;
                    break;
                }
            }
            // If all full, evict the first.
            if !found_free {
                let v = backing[0].key;
                backing[0] = Entry {
                    key,
                    value: key * 10,
                    valid: true,
                };
                evict_ch.try_send(v).expect("evict");
            }
            metrics.inc_mark();
            wg.release();
            thread_b_hits.inc();
        }
    }

    // ---- 6. Final state ----
    println!("[cache] populated:");
    for (i, e) in backing.iter().enumerate() {
        if e.valid {
            println!("  [{}] key={} value={}", i, e.key, e.value);
        }
    }

    println!("\n[metrics] snapshot: {:?}", metrics.snapshot());
    println!("[padded] thread_a_hits = {}", thread_a_hits.load());
    println!("[padded] thread_b_hits = {}", thread_b_hits.load());
    println!("[misses] total: {}", misses);
    println!("[evictions] drained: {}", evict_ch.len());
    println!("[parking lot] parked: {}", lot.parked_count());

    // ---- 7. Increment cycle counter ----
    metrics.inc_cycle();
    let snap = metrics.snapshot();
    println!("[metrics] after cycle: cycles={}", snap[6]);

    println!("\n=== Cache demo complete ===");
}
