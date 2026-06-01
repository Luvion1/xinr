//! Example: slab allocator in action.
//!
//! Demonstrates fixed-size allocation patterns: 16-byte and 64-byte slabs.
//!
//! Run with: `cargo run --features alloc --example slab_demo`

#![cfg(feature = "alloc")]

use xinr::xgc::colored::{Color, ColoredPtr};
use xinr::xgc::slab::SlabAllocator;
use xinr::xgc::slab::size::SlabSize;

fn main() {
    let mut alloc = SlabAllocator::new(0x1_0000_0000);

    println!("Slab allocator base: 0x{:x}", 0x1_0000_0000_u64);
    println!("Initial slab count: {}", alloc.slab_count());

    // Allocate 16-byte objects.
    let mut small = [0usize; 8];
    for (i, slot) in small.iter_mut().enumerate() {
        let p = alloc.alloc(16).expect("alloc 16");
        *slot = p.addr();
        if i < 4 {
            println!("  small[{i}] @ {:#x}", p.addr());
        }
    }

    // Allocate 64-byte objects.
    let mut large = [0usize; 4];
    for (i, slot) in large.iter_mut().enumerate() {
        let p = alloc.alloc(64).expect("alloc 64");
        *slot = p.addr();
        println!("  large[{i}] @ {:#x}", p.addr());
    }

    println!("\nAfter allocations:");
    println!("  slabs: {}", alloc.slab_count());
    println!("  reserved: {} bytes", alloc.reserved());

    // Show that 16-byte and 64-byte use different slabs.
    println!("\nSize classes:");
    for s in SlabSize::ALL {
        println!("  {:?} -> {} slots/page", s, s.slots_per_page());
    }

    // Free a small object; the slab can be reused.
    println!("\nFreeing 16-byte object at {:#x}...", small[0]);
    alloc.free(small[0] as u64, 16);

    let p2 = alloc.alloc(16).expect("re-alloc");
    println!("  re-allocated @ {:#x}", p2.addr());

    // Use a ColoredPtr for fun.
    let cp = ColoredPtr::new(p2.addr(), Color::White);
    println!(
        "\nColoredPtr: addr={:#x}, color={:?}",
        cp.addr(),
        cp.color()
    );

    println!("\nSlab demo complete.");
}
