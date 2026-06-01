//! Parking lot: a single-threaded registry of parked threads.

use crate::sync::parking::permit::Permit;

const MAX_PARKED: usize = 64;

/// A parked thread: who, with what token, for how long.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParkedThread {
    pub thread_id: u64,
    pub permit: Permit,
    pub token: u64,
    pub parked_at: u64,
    pub active: bool,
}

impl ParkedThread {
    /// Whether the entry is in use.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Parking lot (single-threaded; locking is the host's responsibility).
pub struct ParkingLot {
    slots: [ParkedThread; MAX_PARKED],
    next_token: u64,
}

impl ParkingLot {
    /// Construct a new parking lot.
    pub const fn new() -> Self {
        Self {
            slots: [ParkedThread {
                thread_id: 0,
                permit: Permit::NONE,
                token: 0,
                parked_at: 0,
                active: false,
            }; MAX_PARKED],
            next_token: 1,
        }
    }

    /// Total capacity.
    pub const fn capacity(&self) -> usize {
        MAX_PARKED
    }

    /// Number of parked threads.
    pub fn parked_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_active()).count()
    }

    /// Acquire a new permit. Single-threaded; issues monotonic tokens.
    pub fn acquire_permit(&mut self) -> Permit {
        let p = Permit(self.next_token);
        self.next_token = self.next_token.wrapping_add(1);
        if self.next_token == 0 {
            self.next_token = 1;
        }
        p
    }

    /// Park a thread with a permit. Returns the assigned token.
    /// Returns `None` if the lot is full.
    pub fn park(&mut self, thread_id: u64, permit: Permit, t: u64) -> Option<u64> {
        for slot in self.slots.iter_mut() {
            if !slot.is_active() {
                slot.thread_id = thread_id;
                slot.permit = permit;
                slot.token = permit.0;
                slot.parked_at = t;
                slot.active = true;
                return Some(permit.0);
            }
        }
        None
    }

    /// Unpark a thread by token. Returns the parked thread record.
    pub fn unpark(&mut self, token: u64) -> Option<ParkedThread> {
        for slot in self.slots.iter_mut() {
            if slot.is_active() && slot.token == token {
                let record = *slot;
                *slot = ParkedThread::default();
                return Some(record);
            }
        }
        None
    }

    /// Unpark a thread by its thread id (FIFO order).
    pub fn unpark_thread(&mut self, thread_id: u64) -> Option<ParkedThread> {
        for slot in self.slots.iter_mut() {
            if slot.is_active() && slot.thread_id == thread_id {
                let record = *slot;
                *slot = ParkedThread::default();
                return Some(record);
            }
        }
        None
    }

    /// Peek at a parked thread by token.
    pub fn peek(&self, token: u64) -> Option<&ParkedThread> {
        self.slots
            .iter()
            .find(|s| s.is_active() && s.token == token)
    }

    /// Snapshot of all active tokens (in slot order).
    pub fn active_tokens(&self) -> [Option<u64>; MAX_PARKED] {
        let mut out = [None; MAX_PARKED];
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_active() {
                out[i] = Some(slot.token);
            }
        }
        out
    }
}

impl Default for ParkingLot {
    fn default() -> Self {
        Self::new()
    }
}
