//! Page table tests.

use crate::xgc::page::table::PageTable;

#[test]
fn page_table_has_pages() {
    let pt = PageTable::new();
    assert!(!pt.is_empty());
}

#[test]
fn page_table_marks_range_used() {
    let mut pt = PageTable::new();
    pt.mark_range_used(0x10000, 4096);
    assert!(pt.used_count() > 0);
}
