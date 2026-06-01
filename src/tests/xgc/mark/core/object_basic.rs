//! Object, pressure, and pin tests.

use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::object::header::ObjectHeader;
use crate::xgc::object::layout::{Alignment, SizeClass};
use crate::xgc::object::traits::Trace;
use crate::xgc::pin::PinHandle;
use crate::xgc::pressure::threshold::{PressureConfig, PressureMeter};
use crate::xgc::pressure::trigger::GcTrigger;

// --- object tests ---

#[test]
fn header_magic_valid() {
    let h = ObjectHeader::new(42, SizeClass::S32, Alignment::A8);
    assert!(h.is_valid());
    assert_eq!(h.magic, crate::xgc::object::header::HEADER_MAGIC);
}

#[test]
fn header_color_round_trip() {
    let mut h = ObjectHeader::new(1, SizeClass::S32, Alignment::A8);
    assert_eq!(h.color(), Color::White);
    h.set_color(Color::Grey);
    assert_eq!(h.color(), Color::Grey);
    h.set_color(Color::Black);
    assert_eq!(h.color(), Color::Black);
}

#[test]
fn size_class_picks_correctly() {
    assert_eq!(SizeClass::for_payload(0), SizeClass::S32);
    assert_eq!(SizeClass::for_payload(40), SizeClass::S64);
    assert_eq!(SizeClass::for_payload(100), SizeClass::S128);
    assert_eq!(SizeClass::for_payload(400), SizeClass::S512);
    assert_eq!(SizeClass::for_payload(2000), SizeClass::S2K);
    assert_eq!(SizeClass::for_payload(10_000), SizeClass::Large);
}

#[test]
fn alignment_bytes() {
    assert_eq!(Alignment::A8.bytes(), 8);
    assert_eq!(Alignment::A16.bytes(), 16);
    assert_eq!(Alignment::A64.bytes(), 64);
}

#[test]
fn trace_primitives() {
    let v: u32 = 42;
    let mut visited = 0;
    v.trace(&mut Visited {
        count: &mut visited,
    });
    assert_eq!(visited, 0, "primitives have no references");
}

#[test]
fn header_magic_valid_v2() {
    let h = ObjectHeader::new(1, SizeClass::S32, Alignment::A8);
    assert!(h.is_valid());
}

struct Visited<'a> {
    count: &'a mut usize,
}
impl<'a> crate::xgc::object::traits::Visitor for Visited<'a> {
    fn visit(&mut self, _r: ColoredPtr) {
        *self.count += 1;
    }
}

// --- pressure tests ---

#[test]
fn pressure_meter_tracks_alloc() {
    let mut m = PressureMeter::new();
    m.record_alloc(1024);
    m.record_alloc(2048);
    assert_eq!(m.allocated, 3072);
    m.record_free(512);
    assert_eq!(m.freed, 512);
    m.end_cycle();
    assert_eq!(m.cycles, 1);
    assert_eq!(m.allocated, 0, "allocated resets on cycle end");
}

#[test]
fn pressure_config_default_for() {
    let c = PressureConfig::default_for(1_000_000);
    assert_eq!(c.heap_bytes, 1_000_000);
    assert_eq!(c.trigger_bytes(), 800_000);
}

#[test]
fn gc_trigger_fires_above_threshold() {
    let t = GcTrigger::new();
    let c = PressureConfig::default_for(100);
    let mut m = PressureMeter::new();
    m.record_alloc(50);
    assert!(!t.should_trigger(&c, &m, 0));
    m.record_alloc(40);
    assert!(t.should_trigger(&c, &m, 0));
}

// --- pin tests ---

#[test]
fn pin_and_unpin() {
    let mut gc = Xgc::new(4).unwrap();
    let p = ColoredPtr::new(0x1000, Color::Black);
    let h = gc.pin(p).expect("pin");
    assert!(h.is_valid());
    assert!(gc.unpin(h));
    let h2 = gc.pin(p).expect("repin");
    let h3 = gc.pin(p).expect("repin 2");
    assert!(!gc.unpin(h2), "still pinned");
    assert!(gc.unpin(h3), "all unpinned");
}

#[test]
fn pin_handle_invalid() {
    assert!(!PinHandle::INVALID.is_valid());
}

#[test]
fn pinned_object_skipped_during_relocate() {
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let p = ColoredPtr::new(0x2000, Color::Black);
    gc.pin(p).expect("pin");
    gc.begin_mark().unwrap();
    gc.finish_mark();
    gc.begin_relocate();
    gc.record_move(p, ColoredPtr::new(0x3000, Color::Black))
        .expect("record");
    let stats = gc.finish_relocate();
    assert_eq!(stats.moved, 0, "pinned objects are not relocated");
    gc.shutdown().unwrap();
}
