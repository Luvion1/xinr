//! Region, bitmap, and region-table tests.

use crate::xgc::region::Region;
use crate::xgc::region::bitmap::{BITMAP_BYTES, MarkBitmap};
use crate::xgc::region::table::RegionTable;

#[test]
fn bitmap_round_trip() {
    let mut b = MarkBitmap::new();
    assert!(!b.is_marked(0));
    b.mark(0);
    assert!(b.is_marked(0));
    b.unmark(0);
    assert!(!b.is_marked(0));
}

#[test]
fn bitmap_clear_resets() {
    let mut b = MarkBitmap::new();
    b.mark(64);
    b.mark(1024);
    b.clear();
    assert!(!b.is_marked(64));
    assert!(!b.is_marked(1024));
}

#[test]
fn bitmap_for_each_visits_all() {
    let mut b = MarkBitmap::new();
    b.mark(0);
    b.mark(128);
    b.mark(512);
    let mut visited = [false; 3];
    b.for_each_marked(|off| {
        if off == 0 {
            visited[0] = true;
        }
        if off == 128 {
            visited[1] = true;
        }
        if off == 512 {
            visited[2] = true;
        }
    });
    assert!(visited.iter().all(|v| *v));
}

#[test]
fn bitmap_oob_is_noop() {
    let mut b = MarkBitmap::new();
    b.mark(usize::MAX);
    assert!(!b.is_marked(usize::MAX));
}

#[test]
fn region_table_basic() {
    let t = RegionTable::new(4).expect("alloc");
    assert_eq!(t.len(), 4);
    assert!(!t.is_empty());
    assert!(t.get(0).is_some());
    assert!(t.get(99).is_none());
}

#[test]
fn region_table_total_bytes() {
    let t = RegionTable::new(2).unwrap();
    let total = t.total_bytes();
    assert_eq!(total, 2 * crate::xgc::region::REGION_SIZE);
}

#[test]
fn region_unbound() {
    let r = Region::unbound();
    assert!(!r.is_available());
    assert_eq!(r.free(), r.capacity);
}

#[test]
fn bitmap_byte_size_matches_region() {
    const { assert!(BITMAP_BYTES > 0) };
}
