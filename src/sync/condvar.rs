//! Condition variable: wait until notified.
//!
//! Threads register a `Permit`, then call `wait()`. Another thread calls
//! `notify_one()` to wake one waiter or `notify_all()` to wake all.
//! Implementation is a `ParkingLot` wrapper with explicit permit tracking.

use crate::sync::parking::lot::ParkedThread;
use crate::sync::parking::lot::ParkingLot;
use crate::sync::parking::permit::Permit;

/// Condition variable.
pub struct Condvar {
    lot: ParkingLot,
    name: &'static str,
}

impl Condvar {
    /// Construct a new condvar with a debug name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            lot: ParkingLot::new(),
            name,
        }
    }

    /// Debug name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Number of waiters.
    pub fn waiter_count(&self) -> usize {
        self.lot.parked_count()
    }

    /// Register a waiter with `permit`. Returns the assigned token.
    pub fn register_waiter(&mut self, thread_id: u64, permit: Permit, t: u64) -> Option<u64> {
        self.lot.park(thread_id, permit, t)
    }

    /// Notify one waiter by token. Returns the thread record if found.
    pub fn notify_one(&mut self, token: u64) -> Option<ParkedThread> {
        self.lot.unpark(token)
    }

    /// Notify one waiter by thread id (FIFO order).
    pub fn notify_one_thread(&mut self, thread_id: u64) -> Option<ParkedThread> {
        self.lot.unpark_thread(thread_id)
    }

    /// Notify all waiters. Returns the number woken.
    pub fn notify_all(&mut self) -> usize {
        let mut woken = 0;
        for token in self.lot.active_tokens().iter().filter_map(|x| *x) {
            if self.lot.unpark(token).is_some() {
                woken += 1;
            }
        }
        woken
    }

    /// Acquire a permit for parking.
    pub fn acquire_permit(&mut self) -> Permit {
        self.lot.acquire_permit()
    }
}
