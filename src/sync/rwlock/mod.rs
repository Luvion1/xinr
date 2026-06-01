//! Reader-writer lock. Multiple readers OR one writer, never both.

pub mod guard;
pub mod state;

pub use guard::{ReadGuard, WriteGuard};
pub use state::RwState;

use crate::RuntimeError;

/// Reader-writer lock.
pub struct RwLock {
    state: RwState,
}

impl RwLock {
    /// Construct a new lock.
    pub const fn new() -> Self {
        Self {
            state: RwState::new(),
        }
    }

    /// Try to acquire a read lock. Multiple read guards can coexist.
    pub fn try_read(&self) -> Result<ReadGuard<'_>, RuntimeError> {
        if !self.state.try_read() {
            return Err(RuntimeError::WouldBlock);
        }
        Ok(ReadGuard::new(&self.state))
    }

    /// Try to acquire a write lock.
    pub fn try_write(&mut self, id: u64) -> Result<WriteGuard<'_>, RuntimeError> {
        if !self.state.try_write(id) {
            return Err(RuntimeError::WouldBlock);
        }
        Ok(WriteGuard::new(&mut self.state, id))
    }

    /// Whether the lock is free.
    pub fn is_free(&self) -> bool {
        self.state.is_free()
    }

    /// Number of active readers.
    pub fn reader_count(&self) -> u32 {
        self.state.reader_count()
    }

    /// Writer id if held.
    pub fn writer_id(&self) -> Option<u64> {
        self.state.writer_id()
    }
}

impl Default for RwLock {
    fn default() -> Self {
        Self::new()
    }
}
