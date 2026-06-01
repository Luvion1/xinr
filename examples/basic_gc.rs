//! Example: full XGC lifecycle with multiple regions.
//!
//! Run with: `cargo run --features alloc --example basic_gc`

#![cfg(feature = "alloc")]

use xinr::xgc::Xgc;
use xinr::xgc::colored::{Color, ColoredPtr};

fn main() {
    // Create an XGC with capacity for 16 regions.
    let mut gc = Xgc::new(16).expect("Xgc::new failed");
    gc.init().expect("Xgc::init failed");
    println!("XGC initialized with 16 regions.");

    // Mark phase.
    let epoch = gc.begin_mark().expect("begin_mark");
    println!("Mark epoch = {}", epoch);

    let p1 = ColoredPtr::new(0x1000, Color::White);
    let p2 = ColoredPtr::new(0x2000, Color::White);
    let p3 = ColoredPtr::new(0x3000, Color::Black);

    gc.push_root(p1).expect("push_root p1");
    gc.push_root(p2).expect("push_root p2");
    gc.push_root(p3).expect("push_root p3");

    while let Some(item) = gc.pop_work() {
        println!(
            "  traced object @ {:#x} (color={:?})",
            item.addr(),
            item.color()
        );
    }
    gc.finish_mark();
    println!("Mark phase complete; phase = {}", gc.phase().as_str());

    // Relocate phase.
    gc.begin_relocate();
    let new_p1 = ColoredPtr::new(0x4000, Color::Black);
    gc.record_move(p1, new_p1).expect("record_move");
    let stats = gc.finish_relocate();
    println!("Relocation done: moved={}", stats.moved);

    // Inspect heap.
    println!("Heap size: {} bytes", gc.heap_size());
    println!("Should collect? {}", gc.should_collect());

    // Shutdown.
    gc.shutdown().expect("Xgc::shutdown failed");
    println!("XGC shut down cleanly.");
}
