//! Bounded ring buffer for `BoundedChannel`.
//!
//! Fixed-capacity, no_std, `T` generic. Uses `[Option<T>; N]` for storage
//! so any `T` works without `MaybeUninit` ceremony.

use crate::RuntimeError;

/// Bounded ring buffer. `N` must be non-zero.
pub struct BoundedRing<T, const N: usize> {
    buf: [Option<T>; N],
    head: usize,
    tail: usize,
    len: usize,
    closed: bool,
}

impl<T, const N: usize> BoundedRing<T, N> {
    /// Construct a new ring. Const, no allocation.
    pub const fn new() -> Self {
        assert!(N > 0, "BoundedRing capacity must be non-zero");
        Self {
            buf: [const { None }; N],
            head: 0,
            tail: 0,
            len: 0,
            closed: false,
        }
    }

    /// Capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Current number of items.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the ring is full.
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Whether the ring has been closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Try to push. Returns `Full` if no slot available, `Closed` if closed.
    pub fn try_push(&mut self, v: T) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if self.len == N {
            return Err(RuntimeError::WouldBlock);
        }
        self.buf[self.tail] = Some(v);
        self.tail = (self.tail + 1) % N;
        self.len += 1;
        Ok(())
    }

    /// Try to pop. Returns `Empty` if no item, `Closed` if closed and empty.
    pub fn try_pop(&mut self) -> Result<T, RuntimeError> {
        if self.len == 0 {
            return if self.closed {
                Err(RuntimeError::Closed)
            } else {
                Err(RuntimeError::WouldBlock)
            };
        }
        let v = self.buf[self.head].take().ok_or(RuntimeError::WouldBlock)?;
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Ok(v)
    }

    /// Close the ring. Future pushes fail; pops drain then return `Closed`.
    pub fn close(&mut self) {
        self.closed = true;
    }
}

impl<T, const N: usize> Default for BoundedRing<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
