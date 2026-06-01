//! WaitGroup: wait for a counter to reach zero.

use crate::RuntimeError;

/// WaitGroup: counts down as tasks finish, blocks `wait` until zero.
pub struct WaitGroup {
    count: u32,
    waiting: u32,
}

impl WaitGroup {
    /// Construct a new WaitGroup.
    pub const fn new() -> Self {
        Self {
            count: 0,
            waiting: 0,
        }
    }

    /// Current count.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Number of threads currently waiting.
    pub fn waiting(&self) -> u32 {
        self.waiting
    }

    /// Increment the count by `delta` (typically 1 per task).
    pub fn add(&mut self, delta: u32) {
        self.count += delta;
    }

    /// Decrement the count by 1. Returns `Ok(())` if count reached zero.
    pub fn done(&mut self) -> Result<(), RuntimeError> {
        if self.count == 0 {
            return Err(RuntimeError::WouldBlock);
        }
        self.count -= 1;
        if self.count == 0 {
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Wait until the count reaches zero.
    /// Returns `Ok(())` if count is zero, `WouldBlock` otherwise.
    pub fn wait(&mut self) -> Result<(), RuntimeError> {
        self.waiting += 1;
        if self.count == 0 {
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Decrement waiter count (call after `wait` returns).
    pub fn leave(&mut self) {
        if self.waiting > 0 {
            self.waiting -= 1;
        }
    }
}

impl Default for WaitGroup {
    fn default() -> Self {
        Self::new()
    }
}
