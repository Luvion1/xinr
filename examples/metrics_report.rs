//! Example: metrics reporting and live counter inspection.
//!
//! Run with: `cargo run --features alloc --example metrics_report`
//!
//! Exercises the `Metrics` aggregator and shows live/frees/cycles counts
//! after a few GC cycles. Useful for verifying that counters are not
//! double-counted and that the live count converges.

#![cfg(feature = "alloc")]

use xinr::sync::metrics::Metrics;
use xinr::xgc::Xgc;
use xinr::xgc::colored::{Color, ColoredPtr};

fn main() {
    let mut gc = Xgc::new(8).expect("Xgc::new");
    gc.init().expect("init");
    let metrics = Metrics::new();

    println!("Running 20 GC cycles...\n");

    for cycle in 0..20u64 {
        gc.begin_mark().expect("begin_mark");
        metrics.inc_cycle();
        for j in 0..8u64 {
            gc.push_root(ColoredPtr::new(
                (0x1000 * (cycle + 1) + j * 0x10) as usize,
                Color::White,
            ))
            .expect("push_root");
            metrics.inc_alloc();
        }
        while gc.pop_work().is_some() {
            metrics.inc_mark();
        }
        gc.finish_mark();

        gc.begin_relocate();
        for j in 0..8u64 {
            let old = ColoredPtr::new((0x1000 * (cycle + 1) + j * 0x10) as usize, Color::White);
            let new = ColoredPtr::new((0x2000 * (cycle + 1) + j * 0x10) as usize, Color::Black);
            gc.record_move(old, new).expect("record_move");
            metrics.inc_free();
        }
        gc.finish_relocate();

        if cycle % 5 == 4 {
            let snap = metrics.snapshot();
            println!(
                "cycle {:2}: allocs={:3} frees={:3} live={:3} marks={:3} cycles={:3}",
                cycle + 1,
                snap[0],
                snap[1],
                metrics.live(),
                snap[2],
                snap[6]
            );
        }
    }

    gc.shutdown().expect("shutdown");

    let snap = metrics.snapshot();
    println!("\nFinal:");
    println!("  allocs : {}", snap[0]);
    println!("  frees  : {}", snap[1]);
    println!("  live   : {}", metrics.live());
    println!("  marks  : {}", snap[2]);
    println!("  cycles : {}", snap[6]);
    println!("  errors : {}", snap[7]);

    // Sanity: after 20 cycles of 8 allocs + 8 frees, live should be 0.
    assert_eq!(metrics.live(), 0, "no leaks after relocation");
    assert_eq!(snap[6], 20, "20 cycles recorded");
    println!("\nMetrics report complete (no leaks, all counters consistent).");
}
