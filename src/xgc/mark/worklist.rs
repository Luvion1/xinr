//! Mark worklist: LIFO stack of pointers to trace.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;

/// Maximum number of work items.
pub const WORKLIST_CAP: usize = 1024;

/// Mark worklist.
pub struct Worklist {
    items: [Option<ColoredPtr>; WORKLIST_CAP],
    top: usize,
}

impl Worklist {
    /// Construct an empty worklist.
    pub const fn new() -> Self {
        Self {
            items: [None; WORKLIST_CAP],
            top: 0,
        }
    }

    /// Push one entry.
    pub fn push(&mut self, p: ColoredPtr) -> Result<(), RuntimeError> {
        if self.top >= WORKLIST_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        self.items[self.top] = Some(p);
        self.top += 1;
        Ok(())
    }

    /// Pop the most-recently-pushed entry (LIFO).
    pub fn pop(&mut self) -> Option<ColoredPtr> {
        if self.top == 0 {
            return None;
        }
        self.top -= 1;
        self.items[self.top].take()
    }

    /// Current length.
    pub fn len(&self) -> usize {
        self.top
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        for slot in self.items.iter_mut() {
            *slot = None;
        }
        self.top = 0;
    }
}

impl Default for Worklist {
    fn default() -> Self {
        Self::new()
    }
}
