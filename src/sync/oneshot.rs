//! One-shot channel: single value from sender to receiver.

use crate::RuntimeError;
use crate::sync::channel::ring::BoundedRing;

/// One-shot channel. Capacity is exactly 1.
pub struct Oneshot<T> {
    inner: BoundedRing<T, 1>,
}

impl<T> Oneshot<T> {
    /// Construct a new one-shot channel.
    pub const fn new() -> Self {
        Self {
            inner: BoundedRing::new(),
        }
    }

    /// Send the value. Returns `Closed` if a value was already sent.
    pub fn send(&mut self, v: T) -> Result<(), RuntimeError> {
        self.inner.try_push(v)
    }

    /// Receive the value. Returns `Closed` if no value was ever sent.
    pub fn recv(&mut self) -> Result<T, RuntimeError> {
        self.inner.try_pop()
    }

    /// Whether a value has been sent.
    pub fn is_ready(&self) -> bool {
        !self.inner.is_empty()
    }

    /// Whether the value has been received.
    pub fn is_consumed(&self) -> bool {
        self.inner.is_closed() && self.inner.is_empty()
    }

    /// Close the channel.
    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<T> Default for Oneshot<T> {
    fn default() -> Self {
        Self::new()
    }
}
