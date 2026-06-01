//! SATB buffer: records pre-write values for incremental snapshot.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;

const SATB_CAP: usize = 256;

/// Snapshot-at-the-beginning buffer.
pub struct SatbBuffer {
    entries: [Option<ColoredPtr>; SATB_CAP],
    head: usize,
    count: usize,
}

impl SatbBuffer {
    /// Create an empty buffer.
    pub const fn new() -> Self {
        Self {
            entries: [None; SATB_CAP],
            head: 0,
            count: 0,
        }
    }

    /// Record a pre-write value. Errors if full.
    pub fn record(&mut self, p: ColoredPtr) -> Result<(), RuntimeError> {
        if self.count >= SATB_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        self.entries[(self.head + self.count) % SATB_CAP] = Some(p);
        self.count += 1;
        Ok(())
    }

    /// Drain entries through the closure.
    pub fn drain<F: FnMut(ColoredPtr)>(&mut self, mut f: F) {
        for i in 0..self.count {
            let pos = (self.head + i) % SATB_CAP;
            if let Some(p) = self.entries[pos].take() {
                f(p);
            }
        }
        self.head = 0;
        self.count = 0;
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Default for SatbBuffer {
    fn default() -> Self {
        Self::new()
    }
}
