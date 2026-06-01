//! Ring buffer for GC events.

use crate::xgc::log::event::{EventKind, GcEvent};

const RING_CAP: usize = 256;

/// Ring buffer of recent GC events. Overwrites oldest on overflow.
pub struct EventLog {
    ring: [GcEvent; RING_CAP],
    head: usize,
    count: u64,
}

impl EventLog {
    /// Construct an empty log.
    pub const fn new() -> Self {
        Self {
            ring: [GcEvent::new(EventKind::Init, 0, 0); RING_CAP],
            head: 0,
            count: 0,
        }
    }

    /// Record an event at time `t_ms` with payload `value`.
    pub fn record(&mut self, kind: EventKind, t_ms: u64, value: u64) {
        self.ring[self.head] = GcEvent::new(kind, t_ms, value);
        self.head = (self.head + 1) % RING_CAP;
        self.count += 1;
    }

    /// Total events recorded (including those overwritten).
    pub fn total(&self) -> u64 {
        self.count
    }

    /// Number of events currently in the buffer.
    pub fn len(&self) -> usize {
        RING_CAP.min(self.count as usize)
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Iterate the events in chronological order (oldest first).
    pub fn iter_chrono(&self) -> impl Iterator<Item = &GcEvent> {
        let start = if self.count as usize >= RING_CAP {
            self.head
        } else {
            0
        };
        (0..self.len()).map(move |i| &self.ring[(start + i) % RING_CAP])
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
