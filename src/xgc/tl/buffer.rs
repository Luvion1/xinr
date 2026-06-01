//! Thread-local allocation buffer: bypass locking for common paths.

use crate::xgc::colored::ColoredPtr;

/// A single thread-local allocation buffer (TLA).
///
/// Acts as a small object pool per thread. If the buffer is empty the
/// allocator falls back to the shared heap.
pub struct Tlb {
    slots: [Option<ColoredPtr>; 8],
    head: usize,
}

impl Tlb {
    /// Construct an empty TLA.
    pub const fn new() -> Self {
        Self {
            slots: [None; 8],
            head: 0,
        }
    }

    /// Push a pointer into the buffer. Returns false if full.
    pub fn push(&mut self, p: ColoredPtr) -> bool {
        if self.head >= self.slots.len() {
            return false;
        }
        self.slots[self.head] = Some(p);
        self.head += 1;
        true
    }

    /// Pop the most recent pointer.
    pub fn pop(&mut self) -> Option<ColoredPtr> {
        if self.head == 0 {
            return None;
        }
        self.head -= 1;
        self.slots[self.head].take()
    }

    /// Peek at the most recent pointer without removing it.
    pub fn peek(&self) -> Option<ColoredPtr> {
        if self.head == 0 {
            None
        } else {
            self.slots[self.head - 1]
        }
    }

    /// Number of items currently buffered.
    pub fn len(&self) -> usize {
        self.head
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.head == 0
    }

    /// Whether the buffer is full.
    pub fn is_full(&self) -> bool {
        self.head >= self.slots.len()
    }

    /// Drain all items, calling `f` for each.
    pub fn drain<F: FnMut(ColoredPtr)>(&mut self, mut f: F) {
        while let Some(p) = self.pop() {
            f(p);
        }
    }

    /// Capacity.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

impl Default for Tlb {
    fn default() -> Self {
        Self::new()
    }
}
