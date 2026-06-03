//! Finalization queue tests.

use crate::xgc::colored::{Color, ColoredPtr};
use crate::xgc::finalize::FinalizationQueue;

#[test]
fn queue_enqueues_objects() {
    let mut q = FinalizationQueue::new();
    let id = q.enqueue(ColoredPtr::new(0x1000, Color::White), 1).unwrap();
    assert_eq!(q.len(), 1);
    assert_eq!(id.0, 1);
}

#[test]
fn queue_dequeues_in_order() {
    let mut q = FinalizationQueue::new();
    q.enqueue(ColoredPtr::new(0x1000, Color::White), 1).unwrap();
    q.enqueue(ColoredPtr::new(0x2000, Color::Black), 2).unwrap();
    let first = q.dequeue().unwrap();
    let second = q.dequeue().unwrap();
    assert_eq!(first.object.addr(), 0x1000);
    assert_eq!(second.object.addr(), 0x2000);
}