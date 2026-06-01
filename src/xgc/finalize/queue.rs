//! Finalization queue: deferred destructors for objects about to be reclaimed.
//!
//! When the GC determines that an object is unreachable, it does not free
//! the memory immediately. Instead, the object's finalizer (if any) is
//! pushed onto the finalization queue. The user-space runtime drains the
//! queue and runs the destructors out-of-band of the GC cycle.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;

pub const QUEUE_CAP: usize = 128;

/// Opaque finalizer callback id. The actual destructor is dispatched by
/// the runtime; XGC only knows the object pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalizerId(pub u32);

/// One entry in the finalization queue.
#[derive(Debug, Clone, Copy)]
pub struct FinalizeEntry {
    pub object: ColoredPtr,
    pub type_id: u16,
    pub id: FinalizerId,
}

/// Fixed-capacity finalization queue.
pub struct FinalizationQueue {
    entries: [FinalizeEntry; QUEUE_CAP],
    head: usize,
    tail: usize,
    count: usize,
    next_id: u32,
}

impl FinalizationQueue {
    /// Construct an empty queue.
    pub const fn new() -> Self {
        Self {
            entries: [FinalizeEntry {
                object: ColoredPtr::new(0, crate::xgc::colored::Color::White),
                type_id: 0,
                id: FinalizerId(0),
            }; QUEUE_CAP],
            head: 0,
            tail: 0,
            count: 0,
            next_id: 1,
        }
    }

    /// Enqueue an object for finalization.
    ///
    /// # Errors
    ///
    /// Returns `StackOverflow` if the queue is full.
    pub fn enqueue(
        &mut self,
        object: ColoredPtr,
        type_id: u16,
    ) -> Result<FinalizerId, RuntimeError> {
        if self.count >= QUEUE_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        let id = FinalizerId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.entries[self.tail] = FinalizeEntry {
            object,
            type_id,
            id,
        };
        self.tail = (self.tail + 1) % QUEUE_CAP;
        self.count += 1;
        Ok(id)
    }

    /// Dequeue the next finalizer, if any.
    pub fn dequeue(&mut self) -> Option<FinalizeEntry> {
        if self.count == 0 {
            return None;
        }
        let entry = self.entries[self.head];
        self.head = (self.head + 1) % QUEUE_CAP;
        self.count -= 1;
        Some(entry)
    }

    /// Number of pending finalizers.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Default for FinalizationQueue {
    fn default() -> Self {
        Self::new()
    }
}
