//! Diagnostics, finalize, and weak ref tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::diagnostics::dump::{HeapDump, dump_region_table};
use crate::xgc::diagnostics::stats::GcStats;
use crate::xgc::finalize::queue::FinalizationQueue;
use crate::xgc::finalize::weak::{WeakTable, try_upgrade};
use crate::xgc::region::table::RegionTable;

// --- diagnostics tests ---

#[test]
fn stats_starts_zero() {
    let s = GcStats::new();
    assert_eq!(s.bytes_allocated, 0);
    assert_eq!(s.cycles, 0);
}

#[test]
fn stats_records_alloc_and_free() {
    let mut s = GcStats::new();
    s.record_alloc(1024);
    s.record_alloc(512);
    s.record_free(256);
    assert_eq!(s.bytes_allocated, 1536);
    assert_eq!(s.bytes_freed, 256);
    assert_eq!(s.live_bytes(), 1280);
}

#[test]
fn stats_updates_peak() {
    let mut s = GcStats::new();
    s.update_peak(1000);
    s.update_peak(500);
    assert_eq!(s.peak_live_bytes, 1000, "peak is monotonic");
    s.update_peak(2000);
    assert_eq!(s.peak_live_bytes, 2000);
}

#[test]
fn dump_writes_lines() {
    let mut d = HeapDump::new();
    assert_eq!(d.line_count(), 0);
    d.write_line("hello");
    d.write_line("world");
    assert_eq!(d.line_count(), 2);
    assert_eq!(d.get_line(0), b"hello");
    assert_eq!(d.get_line(1), b"world");
}

#[test]
fn dump_truncates_long_lines() {
    let mut d = HeapDump::new();
    d.write_line(&"x".repeat(200));
    assert_eq!(d.get_line(0).len(), 64, "truncated to DUMP_LINE_LEN");
}

#[test]
fn dump_region_table_writes_summary() {
    let t = RegionTable::new(4).unwrap();
    let mut d = HeapDump::new();
    dump_region_table(&t, &mut d);
    assert!(d.line_count() > 0);
    let header = core::str::from_utf8(d.get_line(0)).unwrap();
    assert!(header.contains("Region"));
}

// --- finalize tests ---

#[test]
fn finalize_queue_enqueue_dequeue() {
    let mut q = FinalizationQueue::new();
    let p = ColoredPtr::new(0x1000, Color::Black);
    let id1 = q.enqueue(p, 42).expect("enq1");
    let id2 = q.enqueue(p, 42).expect("enq2");
    assert_ne!(id1, id2);
    assert_eq!(q.len(), 2);
    let e1 = q.dequeue().unwrap();
    let e2 = q.dequeue().unwrap();
    assert_eq!(e1.id, id1);
    assert_eq!(e2.id, id2);
    assert!(q.is_empty());
}

#[test]
fn finalize_queue_overflow() {
    let mut q = FinalizationQueue::new();
    for _ in 0..128 {
        q.enqueue(ColoredPtr::new(0x100, Color::Black), 1).unwrap();
    }
    assert!(q.enqueue(ColoredPtr::new(0x100, Color::Black), 1).is_err());
}

#[test]
fn weak_ref_create_and_upgrade() {
    let mut t = WeakTable::new();
    let p = ColoredPtr::new(0x2000, Color::Black);
    let w = t.create(p).expect("create");
    assert!(t.contains(w));
    let up = try_upgrade(&t, w).expect("upgrade alive");
    assert_eq!(up, p);
}

#[test]
fn weak_ref_invalidate() {
    let mut t = WeakTable::new();
    let p = ColoredPtr::new(0x3000, Color::Black);
    let w = t.create(p).expect("create");
    let n = t.invalidate(p);
    assert_eq!(n, 1);
    assert!(!t.contains(w));
    assert!(try_upgrade(&t, w).is_err());
}
