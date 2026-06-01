//! Example: GC relocation + load barrier pipeline.
//!
//! Run with: `cargo run --features alloc --example gc_relocate_bench`
//!
//! Demonstrates the full mark -> relocate -> barrier -> stats cycle of
//! the XGC subsystem. Useful as a smoke test of the relocation path and
//! as a reproducible performance check.

#![cfg(feature = "alloc")]

use xinr::xgc::Xgc;
use xinr::xgc::barrier::load::LoadBarrier;
use xinr::xgc::colored::{Color, ColoredPtr};

fn main() {
    const CYCLES: u64 = 60;
    const ROOTS_PER_CYCLE: u64 = 16;

    let mut gc = Xgc::new(64).expect("Xgc::new");
    gc.init().expect("init");

    println!(
        "XGC relocation pipeline: {} cycles x {} roots\n",
        CYCLES, ROOTS_PER_CYCLE
    );

    let mut total_marked = 0u64;
    let mut total_moved = 0u64;
    let mut total_satb = 0u64;
    let mut total_ref_updates = 0u64;

    for cycle in 0..CYCLES {
        // 1. Mark phase.
        gc.begin_mark().expect("begin_mark");
        for j in 0..ROOTS_PER_CYCLE {
            let raw = 0x10000 * (cycle + 1) + j * 0x100;
            gc.push_root(ColoredPtr::new(raw as usize, Color::White))
                .expect("push_root");
        }
        let mut drained = 0;
        while gc.pop_work().is_some() {
            drained += 1;
        }
        gc.finish_mark();
        total_marked += drained;

        // 2. Relocate phase.
        gc.begin_relocate();
        for j in 0..ROOTS_PER_CYCLE {
            let old = ColoredPtr::new((0x10000 * (cycle + 1) + j * 0x100) as usize, Color::White);
            let new = ColoredPtr::new((0x20000 * (cycle + 1) + j * 0x100) as usize, Color::Black);
            gc.record_move(old, new).expect("record_move");
        }
        let stats = gc.finish_relocate();
        total_moved += stats.moved as u64;

        // 3. SATB: pre-barrier for a few mutator writes per cycle.
        // (For brevity we just count them; a real consumer would drain
        // the buffer back into the mark stack before the next begin_mark.)
        for j in 0..4u64 {
            let raw = 0x10000 * (cycle + 1) + j * 0x100;
            let old = ColoredPtr::new(raw as usize, Color::White);
            gc.satb_record(old).expect("satb_record");
            total_satb += 1;
        }

        // 4. Ref-update barrier: post-barrier after writes.
        for j in 0..4u64 {
            let raw = 0x20000 * (cycle + 1) + j * 0x100;
            let field = ColoredPtr::new(raw as usize, Color::Black);
            gc.ref_update_record(field.addr(), field)
                .expect("ref_update");
            total_ref_updates += 1;
        }
    }

    // 5. Demonstrate a load barrier on a relocated pointer.
    let sample = ColoredPtr::new(0x10001usize, Color::White);
    let _resolved = sample.load();

    gc.shutdown().expect("shutdown");

    println!("Cycles completed  : {}", CYCLES);
    println!("Total marked      : {}", total_marked);
    println!("Total moved       : {}", total_moved);
    println!("SATB records      : {}", total_satb);
    println!("Ref updates       : {}", total_ref_updates);
    println!("\nXGC relocation pipeline finished cleanly.");
}
