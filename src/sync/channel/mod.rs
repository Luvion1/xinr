//! Bounded MPMC channel. Fixed-capacity, no_std, no allocation.

pub mod ring;

use crate::RuntimeError;
use crate::sync::channel::ring::BoundedRing;

/// Bounded multi-producer multi-consumer channel.
///
/// Backed by a `BoundedRing<T, N>`. The `try_send` / `try_recv` methods
/// are non-blocking; blocking variants are the caller's responsibility
/// (parking permits in `sync::inner`).
pub struct BoundedChannel<T, const N: usize> {
    inner: BoundedRing<T, N>,
}

impl<T, const N: usize> BoundedChannel<T, N> {
    /// Construct a new bounded channel.
    pub const fn new() -> Self {
        Self {
            inner: BoundedRing::new(),
        }
    }

    /// Capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Number of buffered items.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Whether full.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Non-blocking send.
    pub fn try_send(&mut self, v: T) -> Result<(), RuntimeError> {
        self.inner.try_push(v)
    }

    /// Non-blocking receive.
    pub fn try_recv(&mut self) -> Result<T, RuntimeError> {
        self.inner.try_pop()
    }

    /// Close the channel.
    pub fn close(&mut self) {
        self.inner.close();
    }

    /// Whether closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl<T, const N: usize> Default for BoundedChannel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
