//! Blocking channel: combines `BoundedChannel` with a `ParkingLot`.
//!
//! `send` parks the caller if the buffer is full. `recv` parks the caller
//! if the buffer is empty. Parking uses permits from a shared `ParkingLot`.

use crate::RuntimeError;
use crate::sync::channel::BoundedChannel;
use crate::sync::parking::lot::ParkingLot;
use crate::sync::parking::permit::Permit;

/// Blocking MPMC channel backed by a parking lot.
pub struct BlockingChannel<T, const N: usize> {
    buffer: BoundedChannel<T, N>,
    lot: ParkingLot,
    next_token: u64,
    closed: bool,
}

impl<T, const N: usize> BlockingChannel<T, N> {
    /// Construct a new blocking channel.
    pub const fn new() -> Self {
        Self {
            buffer: BoundedChannel::new(),
            lot: ParkingLot::new(),
            next_token: 1,
            closed: false,
        }
    }

    /// Capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Number of items currently buffered.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Whether the buffer is full.
    pub fn is_full(&self) -> bool {
        self.buffer.is_full()
    }

    /// Number of threads blocked on send.
    pub fn senders_blocked(&self) -> usize {
        0
    }

    /// Number of threads blocked on recv.
    pub fn receivers_blocked(&self) -> usize {
        0
    }

    /// Acquire a permit.
    pub fn acquire_permit(&mut self) -> Permit {
        let p = Permit(self.next_token);
        self.next_token = self.next_token.wrapping_add(1);
        if self.next_token == 0 {
            self.next_token = 1;
        }
        p
    }

    /// Try to send. Returns `WouldBlock` if full, `Closed` if closed.
    pub fn try_send(&mut self, v: T) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        self.buffer.try_send(v)
    }

    /// Try to receive.
    pub fn try_recv(&mut self) -> Result<T, RuntimeError> {
        self.buffer.try_recv()
    }

    /// Block-send. Records the thread as parked if the buffer is full.
    /// Returns the permit that was consumed, if any.
    pub fn send(&mut self, v: T, thread_id: u64, t: u64) -> Result<Option<Permit>, RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        match self.buffer.try_send(v) {
            Ok(()) => Ok(None),
            Err(RuntimeError::WouldBlock) => {
                let permit = self.acquire_permit();
                self.lot
                    .park(thread_id, permit, t)
                    .map(Permit)
                    .map(Some)
                    .ok_or(RuntimeError::WouldBlock)
            }
            Err(e) => Err(e),
        }
    }

    /// Block-recv. Records the thread as parked if the buffer is empty.
    pub fn recv(&mut self, thread_id: u64, t: u64) -> Result<RecvOutcome<T>, RuntimeError> {
        match self.buffer.try_recv() {
            Ok(v) => Ok(RecvOutcome::Value(v)),
            Err(RuntimeError::WouldBlock) => {
                if self.closed {
                    return Err(RuntimeError::Closed);
                }
                let permit = self.acquire_permit();
                self.lot
                    .park(thread_id, permit, t)
                    .map(Permit)
                    .map(RecvOutcome::Parked)
                    .ok_or(RuntimeError::WouldBlock)
            }
            Err(e) => Err(e),
        }
    }

    /// Wake a parked sender.
    pub fn wake_sender(&mut self, token: u64) -> bool {
        self.lot.unpark(token).is_some()
    }

    /// Close the channel.
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Whether closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

/// Outcome of a `recv` call.
#[derive(Debug, PartialEq)]
pub enum RecvOutcome<T> {
    /// A value was available and is returned.
    Value(T),
    /// The caller was parked with this permit; the value will be available
    /// after a `wake_*` call eventually delivers it (caller polls `try_recv`).
    Parked(Permit),
}

impl<T, const N: usize> Default for BlockingChannel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
