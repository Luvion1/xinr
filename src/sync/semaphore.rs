//! Counting semaphore: at most `permits` holders at any time.

use crate::RuntimeError;

/// Counting semaphore.
pub struct Semaphore {
    permits: u32,
    max: u32,
    waiting: u32,
}

impl Semaphore {
    /// Construct a new semaphore with `permits` available slots.
    pub const fn new(permits: u32) -> Self {
        Self {
            permits,
            max: permits,
            waiting: 0,
        }
    }

    /// Current available permits.
    pub fn permits(&self) -> u32 {
        self.permits
    }

    /// Maximum permits ever issued.
    pub fn max(&self) -> u32 {
        self.max
    }

    /// Acquire one permit. Returns `WouldBlock` if none available.
    pub fn try_acquire(&mut self) -> Result<(), RuntimeError> {
        if self.permits == 0 {
            return Err(RuntimeError::WouldBlock);
        }
        self.permits -= 1;
        Ok(())
    }

    /// Release one permit. Returns `Closed` if at max.
    pub fn release(&mut self) -> Result<(), RuntimeError> {
        if self.permits >= self.max {
            return Err(RuntimeError::Closed);
        }
        self.permits += 1;
        Ok(())
    }

    /// Number of threads currently waiting (best-effort).
    pub fn waiting(&self) -> u32 {
        self.waiting
    }

    /// Record a thread as waiting.
    pub fn record_wait(&mut self) {
        self.waiting += 1;
    }

    /// Record a thread leaving the wait queue.
    pub fn record_leave(&mut self) {
        if self.waiting > 0 {
            self.waiting -= 1;
        }
    }
}
