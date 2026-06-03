//! Object header tests.

use crate::xgc::object::ObjectHeader;
use crate::xgc::object::layout::{Alignment, SizeClass};

#[test]
fn header_validates_magic() {
    let hdr = ObjectHeader::new(1, SizeClass::S32, Alignment::A8);
    assert!(hdr.is_valid());
}

#[test]
fn header_color_can_be_set() {
    let mut hdr = ObjectHeader::new(1, SizeClass::S32, Alignment::A8);
    hdr.set_color(crate::xgc::colored::Color::Black);
    assert_eq!(hdr.color(), crate::xgc::colored::Color::Black);
}