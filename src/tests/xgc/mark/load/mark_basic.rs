//! Mark phase and worklist tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::mark::phase::{MarkPhase, PhaseCell};
use crate::xgc::mark::worklist::Worklist;

#[test]
fn worklist_starts_empty() {
    let w = Worklist::new();
    assert!(w.is_empty());
    assert_eq!(w.len(), 0);
}

#[test]
fn worklist_push_pop_lifo() {
    let mut w = Worklist::new();
    let a = ColoredPtr::new(0x100, Color::White);
    let b = ColoredPtr::new(0x200, Color::White);
    w.push(a).unwrap();
    w.push(b).unwrap();
    assert_eq!(w.pop(), Some(b), "LIFO order");
    assert_eq!(w.pop(), Some(a));
    assert_eq!(w.pop(), None);
}

#[test]
fn phase_cell_cas() {
    let p = PhaseCell::new();
    assert!(p.cas(MarkPhase::Idle, MarkPhase::Marking));
    assert_eq!(p.load(), MarkPhase::Marking);
    assert!(!p.cas(MarkPhase::Idle, MarkPhase::Idle));
    p.store(MarkPhase::Idle);
    assert_eq!(p.load(), MarkPhase::Idle);
}

#[test]
fn phase_strings() {
    assert_eq!(MarkPhase::Idle.as_str(), "idle");
    assert_eq!(MarkPhase::Marking.as_str(), "marking");
    assert_eq!(MarkPhase::Relocating.as_str(), "relocating");
}
