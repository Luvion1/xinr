//! Event log tests.

use crate::xgc::log::event::EventKind;
use crate::xgc::log::ring::EventLog;

#[test]
fn event_log_records_events() {
    let mut log = EventLog::new();
    log.record(EventKind::Init, 0, 0);
    log.record(EventKind::MarkStart, 10, 0);
    assert_eq!(log.len(), 2);
    assert_eq!(log.total(), 2);
}

#[test]
fn event_log_ring_wraps() {
    let mut log = EventLog::new();
    for i in 0..300 {
        log.record(EventKind::Init, i, 0);
    }
    assert_eq!(log.len(), 256);
    assert_eq!(log.total(), 300);
}