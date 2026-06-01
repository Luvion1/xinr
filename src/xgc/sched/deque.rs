//! Per-worker double-ended task queue (Chase-Lev style, simplified).
//!
//! Each worker has a local deque. The worker pushes/pops from one end
//! (LIFO for cache locality) and other workers steal from the other end
//! (FIFO for fairness).

use crate::xgc::colored::ColoredPtr;

const DEQUE_CAP: usize = 32;

/// Per-worker deque.
pub struct WorkDeque {
    buf: [Option<ColoredPtr>; DEQUE_CAP],
    /// Bottom: index of the next slot to push (LIFO end).
    bottom: usize,
    /// Top: index of the next slot to steal (FIFO end). Uses saturating
    /// arithmetic since we never have more than DEQUE_CAP items.
    top: usize,
}

impl WorkDeque {
    /// Construct an empty deque.
    pub const fn new() -> Self {
        Self {
            buf: [None; DEQUE_CAP],
            bottom: 0,
            top: 0,
        }
    }

    /// Local push (LIFO). Returns false if full.
    pub fn push(&mut self, p: ColoredPtr) -> bool {
        if self.bottom >= DEQUE_CAP {
            return false;
        }
        self.buf[self.bottom] = Some(p);
        self.bottom += 1;
        true
    }

    /// Local pop (LIFO). Returns the most recently pushed item.
    pub fn pop(&mut self) -> Option<ColoredPtr> {
        if self.bottom == 0 || self.bottom <= self.top {
            return None;
        }
        self.bottom -= 1;
        let item = self.buf[self.bottom].take();
        if self.top > self.bottom {
            self.top = self.bottom;
        }
        item
    }

    /// Remote steal (FIFO). Returns the oldest item.
    pub fn steal(&mut self) -> Option<ColoredPtr> {
        if self.top >= self.bottom {
            return None;
        }
        let item = self.buf[self.top].take();
        self.top += 1;
        if self.top >= self.bottom {
            self.top = self.bottom;
        }
        item
    }

    /// Length of the deque.
    pub fn len(&self) -> usize {
        self.bottom.saturating_sub(self.top)
    }

    /// Whether the deque is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Capacity.
    pub fn capacity(&self) -> usize {
        DEQUE_CAP
    }
}

impl Default for WorkDeque {
    fn default() -> Self {
        Self::new()
    }
}
