//! Micro-benchmarks for hot paths in `xinr`.
//!
//! Run with `cargo test --features alloc -- --ignored bench` (benchmarks
//! are marked `#[ignore]` to keep `cargo test` fast).

use crate::sync::channel::BoundedChannel;
use crate::sync::spsc::SpscChannel;
#[cfg(feature = "alloc")]
use crate::xgc::Xgc;
#[cfg(feature = "alloc")]
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
#[ignore = "benchmark"]
fn bench_channel_try_send_recv() {
    let mut ch: BoundedChannel<u32, 64> = BoundedChannel::new();
    crate::bench!("channel_try_send_recv", {
        for i in 0..1000u32 {
            ch.try_send(i).unwrap();
            let _ = ch.try_recv().unwrap();
        }
    });
}

#[test]
#[ignore = "benchmark"]
fn bench_spsc_single_thread() {
    let mut ch: SpscChannel<u32, 64> = SpscChannel::new();
    crate::bench!("spsc_single_thread", {
        for i in 0..1000u32 {
            ch.try_send(i).unwrap();
            let _ = ch.try_recv().unwrap();
        }
    });
}

#[test]
#[ignore = "benchmark"]
fn bench_overhead_only() {
    crate::bench!("empty_loop", {
        let mut sum: u64 = 0;
        for i in 0..1000u64 {
            sum = sum.wrapping_add(i);
        }
        assert_eq!(sum, 499_500);
    });
}

#[test]
fn time_it_returns_result() {
    let (ticks, value) = crate::time_it!(2u32 + 3u32);
    assert_eq!(value, 5);
    // Elapsed is at least 1 (counter advances on every call).
    assert!(ticks >= 1);
}

#[test]
#[ignore = "benchmark"]
fn bench_channel_group() {
    let _ = crate::bench_group!("channel_group", {
        "send_then_recv" => {
            let mut ch: BoundedChannel<u32, 64> = BoundedChannel::new();
            for i in 0..1000u32 {
                let _ = ch.try_send(i);
                if i % 2 == 0 { let _ = ch.try_recv(); }
            }
            for _ in 0..500 { let _ = ch.try_recv(); }
        },
        "round_trip" => {
            let mut ch: BoundedChannel<u32, 64> = BoundedChannel::new();
            for i in 0..1000u32 {
                ch.try_send(i).unwrap();
                let _ = ch.try_recv().unwrap();
            }
        },
        "full_drain" => {
            let mut ch: BoundedChannel<u32, 1024> = BoundedChannel::new();
            for i in 0..1000u32 { ch.try_send(i).unwrap(); }
            for _ in 0..1000 { let _ = ch.try_recv().unwrap(); }
        },
    });
}

#[cfg(feature = "alloc")]
#[test]
#[ignore = "benchmark"]
fn bench_xgc_mark_drain() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    crate::bench!("xgc_mark_drain", {
        for i in 0..50u64 {
            gc.begin_mark().unwrap();
            for j in 0..8u64 {
                gc.push_root(ColoredPtr::new(
                    (0x10000 * (i + 1) + j * 0x100) as usize,
                    Color::White,
                ))
                .unwrap();
            }
            while gc.pop_work().is_some() {}
            gc.finish_mark();
        }
    });
    gc.shutdown().unwrap();
}
