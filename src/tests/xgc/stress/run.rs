//! End-to-end stress test: many GC cycles, allocations, and relocations.

extern crate alloc;
use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};
use alloc::vec::Vec;

#[test]
fn many_cycles() {
    let mut gc = Xgc::new(32).unwrap();
    gc.init().unwrap();
    let mut total_moved = 0;
    for i in 0..50 {
        let epoch = gc.begin_mark().unwrap();
        assert!(epoch > 0);
        // Push 8 roots.
        for j in 0..8u64 {
            gc.push_root(ColoredPtr::new(
                (0x10000 * (i as u64 + 1) + j * 0x100) as usize,
                Color::White,
            ))
            .unwrap();
        }
        // Drain work.
        let mut drained = 0;
        while gc.pop_work().is_some() {
            drained += 1;
        }
        assert_eq!(drained, 8);
        gc.finish_mark();

        // Relocate all of them.
        gc.begin_relocate();
        for j in 0..8u64 {
            let old = ColoredPtr::new(
                (0x10000 * (i as u64 + 1) + j * 0x100) as usize,
                Color::White,
            );
            let new = ColoredPtr::new(
                (0x20000 * (i as u64 + 1) + j * 0x100) as usize,
                Color::Black,
            );
            gc.record_move(old, new).unwrap();
        }
        let stats = gc.finish_relocate();
        total_moved += stats.moved;
    }
    assert_eq!(total_moved, 50 * 8);
    gc.shutdown().unwrap();
}

#[test]
fn pressure_trigger() {
    use crate::xgc::pressure::threshold::PressureMeter;
    let mut m = PressureMeter::new();
    m.record_alloc(500);
    m.record_alloc(400);
    assert!(m.live() > 0);
    m.end_cycle();
    assert_eq!(m.live(), 0);
}

#[test]
fn page_table_round_trip() {
    use crate::xgc::page::align::page_round_up;
    assert_eq!(page_round_up(0), 0);
    assert_eq!(page_round_up(1), 4096);
    assert_eq!(page_round_up(4096), 4096);
    assert_eq!(page_round_up(4097), 8192);
    assert_eq!(page_round_up(1_000_000), 1_003_520);
}

#[test]
fn stress_color_roundtrip() {
    let mut items: Vec<ColoredPtr> = Vec::new();
    for i in 0..1000u64 {
        let c = match i % 3 {
            0 => Color::White,
            1 => Color::Black,
            _ => Color::Grey,
        };
        items.push(ColoredPtr::new(((i + 1) * 4096) as usize, c));
    }
    for (i, p) in items.iter().enumerate() {
        assert_eq!(p.addr(), ((i as u64 + 1) * 4096) as usize);
    }
}
