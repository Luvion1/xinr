//! Waker: bridges the timer wheel to the parking lot.
//!
//! A `Waker` is a token + deadline. When the wheel's `advance` fires the
//! token, the waker looks up the corresponding parked thread in the lot
//! and unpark it.

use crate::sync::parking::lot::{ParkedThread, ParkingLot};
use crate::sync::timer::TimerWheel;

/// Waker registry: maps timer tokens to parking-lot entries.
pub struct Waker {
    /// Mapping of `timer_token -> (thread_id, parking_lot_token)`.
    /// Linear scan, suitable for small N.
    entries: [(u64, u64); 64],
    count: u32,
}

impl Waker {
    /// Construct a new waker.
    pub const fn new() -> Self {
        Self {
            entries: [(0, 0); 64],
            count: 0,
        }
    }

    /// Number of active entries.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Register a waker: when the timer wheel fires `timer_token`, the
    /// thread parked with `parking_token` will be unparked.
    pub fn register(&mut self, timer_token: u64, parking_token: u64) -> bool {
        if (self.count as usize) >= self.entries.len() {
            return false;
        }
        // Replace any existing entry for this timer_token.
        for entry in self.entries[..self.count as usize].iter_mut() {
            if entry.0 == timer_token {
                entry.1 = parking_token;
                return true;
            }
        }
        self.entries[self.count as usize] = (timer_token, parking_token);
        self.count += 1;
        true
    }

    /// Cancel a waker by timer token.
    pub fn cancel(&mut self, timer_token: u64) -> bool {
        for i in 0..self.count as usize {
            if self.entries[i].0 == timer_token {
                self.entries[i] = *self.entries.last().unwrap_or(&(0, 0));
                self.count = self.count.saturating_sub(1);
                return true;
            }
        }
        false
    }

    /// Handle a fired timer token: find and unpark the associated thread.
    pub fn fire(&mut self, timer_token: u64, lot: &mut ParkingLot) -> Option<ParkedThread> {
        for i in 0..self.count as usize {
            if self.entries[i].0 == timer_token {
                let parking_token = self.entries[i].1;
                self.entries[i] = *self.entries.last().unwrap_or(&(0, 0));
                self.count = self.count.saturating_sub(1);
                return lot.unpark(parking_token);
            }
        }
        None
    }

    /// Drive the timer wheel and fire all due wakers into the lot.
    /// Returns the number of threads woken.
    pub fn drive(&mut self, wheel: &mut TimerWheel, lot: &mut ParkingLot, now: u64) -> usize {
        let fired = wheel.advance(now);
        let mut woke = 0;
        for timer_token in fired.iter().filter_map(|x| *x) {
            if self.fire(timer_token, lot).is_some() {
                woke += 1;
            }
        }
        woke
    }
}

impl Default for Waker {
    fn default() -> Self {
        Self::new()
    }
}
