//! Region table tests.

use crate::xgc::region::table::RegionTable;

#[test]
fn region_table_creates_regions() {
    let rt = RegionTable::new(8).unwrap();
    assert!(!rt.is_empty());
}

#[test]
fn region_table_total_bytes() {
    let rt = RegionTable::new(4).unwrap();
    assert!(rt.total_bytes() > 0);
}
