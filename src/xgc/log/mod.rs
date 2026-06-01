//! GC event log: ring buffer of timeline events for diagnostics.

pub mod event;
pub mod ring;

pub use event::{EventKind, GcEvent};
pub use ring::EventLog;
