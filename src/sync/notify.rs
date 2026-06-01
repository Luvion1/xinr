//! Notify: simple event flag for one-shot signaling.
//!
//! A `Notify` is a single-bit flag. Callers `notify_one()` to set, and
//! `wait()`/`try_wait()` to read and clear. Suitable for "wake one" /
//! "wake all" semantics without the full condvar machinery.

use crate::RuntimeError;

/// One-shot notification flag.
pub struct Notify {
    flag: bool,
    waiters: u32,
}

impl Notify {
    /// Construct a new (unset) notify.
    pub const fn new() -> Self {
        Self {
            flag: false,
            waiters: 0,
        }
    }

    /// Whether the flag is currently set.
    pub fn is_notified(&self) -> bool {
        self.flag
    }

    /// Number of threads currently waiting.
    pub fn waiters(&self) -> u32 {
        self.waiters
    }

    /// Set the flag.
    pub fn notify_one(&mut self) {
        self.flag = true;
    }

    /// Wait until the flag is set. If already set, returns immediately.
    /// The flag is NOT cleared by `wait`.
    pub fn wait(&mut self) -> Result<(), RuntimeError> {
        self.waiters += 1;
        if self.flag {
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Try to wait: returns `Ok(())` if flagged, `WouldBlock` if not.
    /// Also increments the waiter count.
    pub fn try_wait(&mut self) -> Result<(), RuntimeError> {
        self.waiters += 1;
        if self.flag {
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Manually clear the flag.
    pub fn clear(&mut self) {
        self.flag = false;
    }

    /// Notify all waiters (just sets the flag for the next waiter).
    pub fn notify_all(&mut self) {
        self.flag = true;
    }

    /// Decrement waiter count (call after `wait` returns).
    pub fn leave(&mut self) {
        if self.waiters > 0 {
            self.waiters -= 1;
        }
    }
}

impl Default for Notify {
    fn default() -> Self {
        Self::new()
    }
}
