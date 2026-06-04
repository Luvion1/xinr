//! Heap allocator integration tests.

use crate::xgc::colored::Color;
use crate::xgc::heap::Xgc;
use crate::xgc::object::ObjectHeader;
use crate::xgc::object::layout::Alignment;

#[test]
fn allocate_returns_non_null() {
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let hdr = ObjectHeader::new(
        1,
        crate::xgc::object::layout::SizeClass::S64,
        Alignment::A64,
    );
    let ptr = gc.allocate(&hdr, 48).expect("allocate failed");
    assert!(!ptr.is_null());
}

#[test]
fn allocate_writes_header() {
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let hdr = ObjectHeader::new(
        99,
        crate::xgc::object::layout::SizeClass::S64,
        Alignment::A64,
    );
    let ptr = gc.allocate(&hdr, 32).unwrap();
    unsafe {
        let stored = *(ptr.sub(core::mem::size_of::<ObjectHeader>()) as *const ObjectHeader);
        assert_eq!(stored.id, 99);
    }
}

#[test]
fn sweep_frees_white_objects() {
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let hdr = ObjectHeader::new(
        1,
        crate::xgc::object::layout::SizeClass::S64,
        Alignment::A64,
    );
    let ptr = gc.allocate(&hdr, 32).expect("allocate failed");
    unsafe {
        let base = ptr.sub(core::mem::size_of::<ObjectHeader>());
        let hdr_slot = base as *mut ObjectHeader;
        (*hdr_slot).set_color(Color::White);
    }
    let (freed, _) = gc.sweep();
    assert_eq!(freed, 1);
}

#[test]
fn sweep_keeps_black_objects() {
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let hdr_grey = ObjectHeader::new(
        1,
        crate::xgc::object::layout::SizeClass::S64,
        Alignment::A64,
    );
    let ptr = gc.allocate(&hdr_grey, 32).expect("allocate failed");
    unsafe {
        let base = ptr.sub(core::mem::size_of::<ObjectHeader>());
        let hdr_slot = base as *mut ObjectHeader;
        (*hdr_slot).set_color(Color::Black);
    }
    let (freed, _) = gc.sweep();
    assert_eq!(freed, 0);
}
