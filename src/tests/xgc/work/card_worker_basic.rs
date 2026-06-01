//! Card table and worker tests.

use crate::xgc::Xgc;
use crate::xgc::card::byte::{CARD_SIZE, CARDS_PER_REGION, CardByte, CardState};
use crate::xgc::card::table::CardTable;
use crate::xgc::worker::signal::{GcSignal, WorkOrder};
use crate::xgc::worker::thread::GcWorker;

// --- card tests ---

#[test]
fn card_constants_reasonable() {
    const {
        assert!(CARD_SIZE > 0);
        assert!(CARD_SIZE <= 4096);
        assert!(CARDS_PER_REGION > 0);
    }
}

#[test]
fn card_byte_round_trip() {
    let c = CardByte::new();
    assert_eq!(c.load(), CardState::Clean);
    c.mark_dirty();
    assert_eq!(c.load(), CardState::Dirty);
    c.store(CardState::Clean);
    assert_eq!(c.load(), CardState::Clean);
}

#[test]
fn card_table_starts_clean() {
    let t = CardTable::new();
    assert!(!t.is_dirty_at(0));
    assert!(!t.is_dirty_at(1024));
}

#[test]
fn card_table_mark_and_iterate() {
    let t = CardTable::new();
    t.mark_dirty_at(0);
    t.mark_dirty_at(2 * CARD_SIZE);
    t.mark_dirty_at(5 * CARD_SIZE);
    let mut visited = [false; 3];
    let mut i = 0;
    t.for_each_dirty(|_idx| {
        if i < 3 {
            visited[i] = true;
        }
        i += 1;
    });
    assert!(visited.iter().all(|v| *v));
    assert_eq!(i, 3);
}

#[test]
fn card_table_oob_safe() {
    let t = CardTable::new();
    t.mark_dirty_at(usize::MAX);
    assert!(!t.is_dirty_at(usize::MAX));
}

#[test]
fn card_table_clear() {
    let t = CardTable::new();
    t.mark_dirty_at(0);
    t.mark_dirty_at(CARD_SIZE);
    t.clear();
    assert!(!t.is_dirty_at(0));
    assert!(!t.is_dirty_at(CARD_SIZE));
}

// --- worker tests ---

#[test]
fn signal_starts_idle() {
    let s = GcSignal::new();
    assert_eq!(s.order(), WorkOrder::None);
    assert!(!s.is_wake());
}

#[test]
fn signal_request_and_consume() {
    let s = GcSignal::new();
    s.request(WorkOrder::Mark);
    assert!(s.is_wake());
    assert_eq!(s.order(), WorkOrder::Mark);
    let o = s.consume();
    assert_eq!(o, WorkOrder::Mark);
    assert!(!s.is_wake());
}

#[test]
fn signal_order_from_byte() {
    assert_eq!(WorkOrder::from_byte(0), WorkOrder::None);
    assert_eq!(WorkOrder::from_byte(1), WorkOrder::Mark);
    assert_eq!(WorkOrder::from_byte(2), WorkOrder::Sweep);
    assert_eq!(WorkOrder::from_byte(3), WorkOrder::Relocate);
    assert_eq!(WorkOrder::from_byte(4), WorkOrder::Shutdown);
    assert_eq!(WorkOrder::from_byte(99), WorkOrder::None);
}

#[test]
fn worker_process_one_mark() {
    let w = GcWorker::new();
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    w.request_mark();
    let processed = w.process_one(&mut gc);
    assert_eq!(processed, WorkOrder::Mark);
    assert_eq!(gc.phase().as_str(), "marking");
    gc.finish_mark();
    gc.shutdown().unwrap();
}

#[test]
fn worker_process_none_when_idle() {
    let w = GcWorker::new();
    let mut gc = Xgc::new(4).unwrap();
    gc.init().unwrap();
    let processed = w.process_one(&mut gc);
    assert_eq!(processed, WorkOrder::None);
    gc.shutdown().unwrap();
}
