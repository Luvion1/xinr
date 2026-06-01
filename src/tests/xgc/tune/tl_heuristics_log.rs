//! TL, heuristics, and log tests.

extern crate alloc;
use alloc::vec::Vec;

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::heuristics::sizing::HeapSizing;
use crate::xgc::heuristics::trigger::{CollectionTrigger, Trigger};
use crate::xgc::log::event::EventKind;
use crate::xgc::log::ring::EventLog;
use crate::xgc::tl::buffer::Tlb;
use crate::xgc::tl::context::ThreadCtx;

// --- TLB ---

#[test]
fn tlb_push_pop() {
    let mut t = Tlb::new();
    assert!(t.is_empty());
    assert!(t.push(ColoredPtr::new(0x1000, Color::White)));
    assert!(t.push(ColoredPtr::new(0x2000, Color::Black)));
    assert_eq!(t.len(), 2);
    let p2 = t.pop().unwrap();
    assert_eq!(p2.addr(), 0x2000);
    let p1 = t.pop().unwrap();
    assert_eq!(p1.addr(), 0x1000);
    assert!(t.is_empty());
}

#[test]
fn tlb_capacity_limit() {
    let mut t = Tlb::new();
    for i in 0..8 {
        assert!(t.push(ColoredPtr::new(0x100 + i, Color::White)));
    }
    assert!(t.is_full());
    assert!(!t.push(ColoredPtr::new(0x999, Color::White)));
}

#[test]
fn tlb_peek_doesnt_remove() {
    let mut t = Tlb::new();
    let p = ColoredPtr::new(0x1000, Color::White);
    let expected_addr = p.addr();
    t.push(p);
    assert_eq!(t.peek().unwrap().addr(), expected_addr);
    assert_eq!(t.len(), 1);
}

#[test]
fn tlb_drain() {
    let mut t = Tlb::new();
    t.push(ColoredPtr::new(0x10, Color::White));
    t.push(ColoredPtr::new(0x20, Color::White));
    t.push(ColoredPtr::new(0x30, Color::White));
    let mut total = 0;
    t.drain(|p| total += p.addr());
    assert_eq!(total, 0x10 + 0x20 + 0x30);
    assert!(t.is_empty());
}

#[test]
fn thread_ctx_local_alloc_hit() {
    let mut c = ThreadCtx::new(0);
    c.return_to_local(ColoredPtr::new(0x100, Color::White));
    let p = c.try_local_alloc().unwrap();
    assert_eq!(p.addr(), 0x100);
    assert_eq!(c.cache_hits, 1);
    assert_eq!(c.allocs, 0);
}

#[test]
fn thread_ctx_miss() {
    let mut c = ThreadCtx::new(1);
    assert!(c.try_local_alloc().is_none());
    c.record_alloc();
    c.record_alloc();
    assert_eq!(c.allocs, 2);
    assert_eq!(c.cache_misses, 2);
}

#[test]
fn thread_ctx_hit_rate() {
    let mut c = ThreadCtx::new(0);
    c.return_to_local(ColoredPtr::new(0x1000, Color::White));
    c.try_local_alloc();
    c.record_alloc();
    c.record_alloc();
    let r = c.hit_rate();
    assert!((r - 0.5).abs() < 0.01, "1 hit / 2 allocs = 0.5, got {}", r);
}

// --- heuristics ---

#[test]
fn heap_sizing_grow_decision() {
    let s = HeapSizing::default_conservative();
    assert!(s.should_grow(850, 1000), "80%+ usage should grow");
    assert!(!s.should_grow(500, 1000));
}

#[test]
fn heap_sizing_shrink_decision() {
    let s = HeapSizing::default_conservative();
    assert!(s.should_shrink(200, 1000), "30%- usage should shrink");
    assert!(!s.should_shrink(500, 1000));
}

#[test]
fn heap_sizing_grow_factor() {
    let s = HeapSizing::default_conservative();
    let new_cap = s.grown_capacity(1000);
    assert_eq!(new_cap, 1250);
}

#[test]
fn heap_sizing_aggressive() {
    let s = HeapSizing::default_aggressive();
    assert!(s.should_grow(700, 1000), "aggressive: triggers earlier");
}

#[test]
fn trigger_full() {
    let mut t = CollectionTrigger::new(1000);
    t.update_usage(1000);
    assert_eq!(t.decide(), Trigger::Full);
}

#[test]
fn trigger_high_watermark() {
    let mut t = CollectionTrigger::new(1000);
    t.update_usage(850);
    assert_eq!(t.decide(), Trigger::High);
}

#[test]
fn trigger_promotion() {
    let mut t = CollectionTrigger::new(10_000);
    t.promotion_threshold = 5;
    for _ in 0..5 {
        t.record_promotion();
    }
    assert_eq!(t.decide(), Trigger::Promotion);
}

#[test]
fn trigger_allocation() {
    let mut t = CollectionTrigger::new(10_000);
    t.alloc_threshold = 5;
    for _ in 0..5 {
        t.record_alloc();
    }
    assert_eq!(t.decide(), Trigger::Allocation);
}

#[test]
fn trigger_reset() {
    let mut t = CollectionTrigger::new(10_000);
    for _ in 0..10 {
        t.record_alloc();
    }
    t.reset();
    assert_eq!(t.alloc_count, 0);
}

// --- log ---

#[test]
fn log_record_and_iterate() {
    let mut log = EventLog::new();
    log.record(EventKind::Init, 0, 0);
    log.record(EventKind::MarkStart, 1, 100);
    log.record(EventKind::MarkEnd, 2, 200);
    assert_eq!(log.total(), 3);
    let events: Vec<_> = log.iter_chrono().collect();
    assert_eq!(events[0].kind, EventKind::Init);
    assert_eq!(events[1].kind, EventKind::MarkStart);
}

#[test]
fn log_ring_overwrites() {
    let mut log = EventLog::new();
    for i in 0..300 {
        log.record(EventKind::Alloc, i as u64, i as u64);
    }
    assert_eq!(log.total(), 300);
    assert_eq!(log.len(), 256, "ring holds at most cap entries");
    let last = log.iter_chrono().last().unwrap();
    assert_eq!(last.value, 299);
}

#[test]
fn log_event_label() {
    assert_eq!(EventKind::Init.label(), "init");
    assert_eq!(EventKind::MarkStart.label(), "mark.start");
    assert_eq!(EventKind::Error.label(), "error");
}
