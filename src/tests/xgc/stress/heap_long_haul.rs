//! Long-haul heap stress test: 1000+ GC cycles to detect state leaks.
//!
//! Mark this test with `#[ignore]` so the default `cargo test` stays
//! fast; run explicitly with `cargo test --features alloc --
//! --ignored heap_long_haul`.

use crate::sync::metrics::Metrics;
use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
#[ignore = "long-haul stress; run with --ignored"]
fn heap_long_haul() {
    const CYCLES: u32 = 1000;
    const ROOTS_PER_CYCLE: u32 = 32;

    let mut gc = Xgc::new(16).expect("Xgc::new");
    gc.init().expect("init");
    let metrics = Metrics::new();

    for cycle in 0..CYCLES {
        // 1. Mark phase.
        gc.begin_mark().expect("begin_mark");
        for j in 0..ROOTS_PER_CYCLE {
            let raw = 0x1000_0000u64 + (cycle as u64 * ROOTS_PER_CYCLE as u64) + j as u64;
            gc.push_root(ColoredPtr::new(raw as usize, Color::White))
                .expect("push_root");
        }
        let mut drained = 0u32;
        while gc.pop_work().is_some() {
            drained += 1;
        }
        assert_eq!(drained, ROOTS_PER_CYCLE, "cycle {} drain", cycle);
        gc.finish_mark();
        metrics.inc_mark();

        // 2. Relocate half of the roots.
        gc.begin_relocate();
        let to_move = ROOTS_PER_CYCLE / 2;
        for j in 0..to_move {
            let old = ColoredPtr::new(
                (0x1000_0000u64 + (cycle as u64 * ROOTS_PER_CYCLE as u64) + j as u64) as usize,
                Color::White,
            );
            let new = ColoredPtr::new(
                (0x2000_0000u64 + (cycle as u64 * ROOTS_PER_CYCLE as u64) + j as u64) as usize,
                Color::Black,
            );
            gc.record_move(old, new).expect("record_move");
        }
        let stats = gc.finish_relocate();
        assert_eq!(stats.moved, to_move, "cycle {} moved", cycle);
        metrics.add(0, ROOTS_PER_CYCLE as u64); // allocs
        metrics.add(1, to_move as u64); // frees (relocated away)

        // 3. Periodic pressure check: every 100 cycles, verify metrics
        //    are within bounds (no negative live, no overflow).
        if cycle % 100 == 99 {
            assert!(
                metrics.live() < 100_000_000,
                "live count sane at cycle {}",
                cycle
            );
        }
    }

    gc.shutdown().expect("shutdown");

    let snap = metrics.snapshot();
    assert_eq!(snap[2], CYCLES as u64, "marks = cycles");
    let expected_moved = (CYCLES as u64) * (ROOTS_PER_CYCLE as u64) / 2;
    assert_eq!(snap[1], expected_moved, "frees = expected_moved");
}
