//! Single-producer single-consumer channel.
//!
//! A `SpscChannel<T, N>` has a single sender and a single receiver, no
//! internal locking. `try_send` advances the head; `try_recv` advances
//! the tail. The full/empty flags track state without a count.

use crate::RuntimeError;

/// SPSC bounded channel.
pub struct SpscChannel<T, const N: usize> {
    buf: [Option<T>; N],
    head: usize, // next write
    tail: usize, // next read
    full: bool,
}

impl<T, const N: usize> SpscChannel<T, N> {
    /// Construct a new SPSC channel.
    pub const fn new() -> Self {
        Self {
            buf: [const { None }; N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// Capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        if self.full {
            N
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            N - (self.tail - self.head)
        }
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether full.
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Try to send.
    pub fn try_send(&mut self, v: T) -> Result<(), RuntimeError> {
        if self.full {
            return Err(RuntimeError::WouldBlock);
        }
        self.buf[self.head] = Some(v);
        self.head = (self.head + 1) % N;
        if self.head == self.tail {
            self.full = true;
        }
        Ok(())
    }

    /// Try to receive.
    pub fn try_recv(&mut self) -> Result<T, RuntimeError> {
        if self.is_empty() {
            return Err(RuntimeError::WouldBlock);
        }
        let v = self.buf[self.tail].take().ok_or(RuntimeError::WouldBlock)?;
        self.tail = (self.tail + 1) % N;
        self.full = false;
        Ok(v)
    }
}

impl<T, const N: usize> Default for SpscChannel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-producer single-consumer channel.
pub struct MpscChannel<T, const N: usize> {
    inner: crate::sync::channel::BoundedChannel<T, N>,
}

impl<T, const N: usize> MpscChannel<T, N> {
    /// Construct a new MPSC channel.
    pub const fn new() -> Self {
        Self {
            inner: crate::sync::channel::BoundedChannel::new(),
        }
    }

    /// Capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Try to send (any producer).
    pub fn try_send(&mut self, v: T) -> Result<(), RuntimeError> {
        self.inner.try_send(v)
    }

    /// Try to receive (single consumer).
    pub fn try_recv(&mut self) -> Result<T, RuntimeError> {
        self.inner.try_recv()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Whether full.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Length.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T, const N: usize> Default for MpscChannel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
