//! Reference-update buffer: tracks post-write values for relocation remap.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;

const REF_UPDATE_CAP: usize = 256;

#[derive(Clone, Copy, Debug)]
struct Entry {
    field: usize,
    old: ColoredPtr,
}

/// Field update log.
pub struct RefUpdateBuffer {
    entries: [Option<Entry>; REF_UPDATE_CAP],
    head: usize,
    count: usize,
}

impl RefUpdateBuffer {
    /// Create an empty buffer.
    pub const fn new() -> Self {
        Self {
            entries: [None; REF_UPDATE_CAP],
            head: 0,
            count: 0,
        }
    }

    /// Record a field update.
    pub fn record(&mut self, field: usize, old: ColoredPtr) -> Result<(), RuntimeError> {
        if self.count >= REF_UPDATE_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        let pos = (self.head + self.count) % REF_UPDATE_CAP;
        self.entries[pos] = Some(Entry { field, old });
        self.count += 1;
        Ok(())
    }

    /// Drain entries through the closure.
    pub fn drain<F: FnMut(usize, ColoredPtr)>(&mut self, mut f: F) {
        for i in 0..self.count {
            let pos = (self.head + i) % REF_UPDATE_CAP;
            if let Some(Entry { field, old }) = self.entries[pos].take() {
                f(field, old);
            }
        }
        self.head = 0;
        self.count = 0;
    }
}

impl Default for RefUpdateBuffer {
    fn default() -> Self {
        Self::new()
    }
}
