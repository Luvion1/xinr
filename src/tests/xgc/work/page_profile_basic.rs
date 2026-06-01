//! Page allocator and allocation profiler tests.

use crate::xgc::page::align::{
    PAGE_SIZE, REGION_PAGE_COUNT, is_page_aligned, page_round_down, page_round_up,
};
use crate::xgc::page::descriptor::{PageDescriptor, PageState};
use crate::xgc::page::table::PageTable;
use crate::xgc::profile::AllocProfiler;
use crate::xgc::profile::site::SiteId;
use crate::xgc::profile::stats::SiteStats;

// --- page tests ---

#[test]
fn page_size_is_4k() {
    assert_eq!(PAGE_SIZE, 4096);
}

#[test]
fn page_round_up_basic() {
    assert_eq!(page_round_up(0), 0);
    assert_eq!(page_round_up(1), PAGE_SIZE);
    assert_eq!(page_round_up(PAGE_SIZE), PAGE_SIZE);
    assert_eq!(page_round_up(PAGE_SIZE + 1), 2 * PAGE_SIZE);
}

#[test]
fn page_round_down_basic() {
    assert_eq!(page_round_down(0), 0);
    assert_eq!(page_round_down(1), 0);
    assert_eq!(page_round_down(PAGE_SIZE), PAGE_SIZE);
    assert_eq!(page_round_down(PAGE_SIZE + 100), PAGE_SIZE);
}

#[test]
fn page_aligned_check() {
    assert!(is_page_aligned(0));
    assert!(is_page_aligned(PAGE_SIZE));
    assert!(!is_page_aligned(1));
    assert!(!is_page_aligned(PAGE_SIZE + 1));
}

#[test]
fn page_region_count() {
    assert_eq!(
        REGION_PAGE_COUNT,
        crate::xgc::region::REGION_SIZE / PAGE_SIZE
    );
}

#[test]
fn page_descriptor_lifecycle() {
    let mut p = PageDescriptor::free();
    assert!(p.is_free());
    p.mark_used();
    assert!(p.is_used());
    p.mark_dirty();
    assert!(p.is_used());
    p.mark_free();
    assert!(p.is_free());
}

#[test]
fn page_state_from_byte() {
    assert_eq!(PageState::from_byte(0), PageState::Free);
    assert_eq!(PageState::from_byte(1), PageState::Used);
    assert_eq!(PageState::from_byte(2), PageState::Reserved);
    assert_eq!(PageState::from_byte(3), PageState::Dirty);
    assert_eq!(PageState::from_byte(99), PageState::Free);
}

#[test]
fn page_table_starts_free() {
    let t = PageTable::new();
    assert_eq!(t.used_count(), 0);
    assert!(t.get(0).unwrap().is_free());
}

#[test]
fn page_table_mark_used() {
    let mut t = PageTable::new();
    t.mark_range_used(0, PAGE_SIZE);
    assert_eq!(t.used_count(), 1);
    t.mark_range_used(PAGE_SIZE, 2 * PAGE_SIZE);
    assert_eq!(t.used_count(), 3);
}

#[test]
fn page_table_clear() {
    let mut t = PageTable::new();
    t.mark_range_used(0, 5 * PAGE_SIZE);
    assert_eq!(t.used_count(), 5);
    t.clear();
    assert_eq!(t.used_count(), 0);
}

#[test]
fn page_table_for_each_in() {
    let mut t = PageTable::new();
    t.mark_range_used(0, 2 * PAGE_SIZE);
    t.mark_range_used(4 * PAGE_SIZE, PAGE_SIZE);
    let mut dirty_count = 0;
    t.for_each_in(PageState::Used, |_, _| dirty_count += 1);
    assert_eq!(dirty_count, 3);
}

// --- profile tests ---

#[test]
fn site_id_fresh_unique() {
    let a = SiteId::fresh();
    let b = SiteId::fresh();
    assert_ne!(a, b);
}

#[test]
fn site_id_anon() {
    assert_eq!(SiteId::ANON.0, 0);
}

#[test]
fn site_stats_round_trip() {
    let mut s = SiteStats::new();
    s.record_alloc(100);
    s.record_alloc(200);
    s.record_free(50);
    assert_eq!(s.alloc_count, 2);
    assert_eq!(s.alloc_bytes, 300);
    assert_eq!(s.free_bytes, 50);
    assert_eq!(s.live_bytes(), 250);
}

#[test]
fn profiler_record_and_query() {
    let mut p = AllocProfiler::new();
    let id = SiteId::fresh();
    p.record_alloc(id, 1024);
    p.record_alloc(id, 2048);
    p.record_free(id, 512);
    let s = p.get(id).expect("present");
    assert_eq!(s.alloc_count, 2);
    assert_eq!(s.alloc_bytes, 3072);
    assert_eq!(s.live_bytes(), 2560);
}

#[test]
fn profiler_total_alloc_bytes() {
    let mut p = AllocProfiler::new();
    let id1 = SiteId::fresh();
    let id2 = SiteId::fresh();
    p.record_alloc(id1, 100);
    p.record_alloc(id2, 200);
    assert_eq!(p.total_alloc_bytes(), 300);
    assert_eq!(p.site_count(), 2);
}

#[test]
fn profiler_reset() {
    let mut p = AllocProfiler::new();
    let id = SiteId::fresh();
    p.record_alloc(id, 1000);
    p.reset();
    let s = p.get(id).unwrap();
    assert_eq!(s.alloc_bytes, 0);
}
