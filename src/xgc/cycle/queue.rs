//! Cycle detector worklist: queue of candidate objects to test.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;
use crate::xgc::cycle::stats::CycleCandidate;

const CYCLE_CAP: usize = 64;

/// Bounded FIFO of cycle candidates.
pub struct CycleQueue {
    pub entries: [Option<CycleCandidate>; CYCLE_CAP],
    pub head: usize,
    pub count: usize,
}

impl Default for CycleQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CycleQueue {
    /// Construct an empty queue.
    pub const fn new() -> Self {
        Self {
            entries: [None; CYCLE_CAP],
            head: 0,
            count: 0,
        }
    }

    /// Whether empty.
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Current length.
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Enqueue a candidate.
    pub fn enqueue(&mut self, c: CycleCandidate) -> Result<(), RuntimeError> {
        if self.count >= CYCLE_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        self.entries[self.count] = Some(c);
        self.count += 1;
        Ok(())
    }

    /// Dequeue the oldest candidate.
    pub fn dequeue(&mut self) -> Option<CycleCandidate> {
        if self.count == 0 {
            return None;
        }
        let item = self.entries[self.head].take();
        self.head += 1;
        if self.head == self.count {
            self.head = 0;
            self.count = 0;
        }
        item
    }
}

#[allow(dead_code)]
fn _phantom(_: ColoredPtr) {}
