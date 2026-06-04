//! RwLock state: count of readers (in `UnsafeCell` for shared access) and
//! the id of the writer (if any).

use core::cell::UnsafeCell;

/// RwLock state. The reader count is in an `UnsafeCell` so multiple read
/// guards can coexist (each holding a `&self`).
#[derive(Debug)]
pub struct RwState {
    readers: UnsafeCell<i32>,
    /// Writer id, or 0 if free.
    writer: u64,
}

impl RwState {
    /// Construct an unlocked state.
    pub const fn new() -> Self {
        Self {
            readers: UnsafeCell::new(0),
            writer: 0,
        }
    }

    /// Read the current reader count.
    pub fn reader_count(&self) -> u32 {
        unsafe { *self.readers.get() as u32 }
    }

    /// Writer id, if any.
    pub fn writer_id(&self) -> Option<u64> {
        if self.writer == 0 {
            None
        } else {
            Some(self.writer)
        }
    }

    /// Whether the lock is free.
    pub fn is_free(&self) -> bool {
        unsafe { *self.readers.get() == 0 && self.writer == 0 }
    }

    /// Whether the lock is held by readers.
    pub fn is_read(&self) -> bool {
        unsafe { *self.readers.get() > 0 }
    }

    /// Whether the lock is held by a writer.
    pub fn is_write(&self) -> bool {
        self.writer != 0
    }

    /// Try to acquire a read lock. `&self` so multiple read guards can coexist.
    /// Returns false if a writer holds it or the reader count is saturated.
    pub fn try_read(&self) -> bool {
        if self.writer != 0 {
            return false;
        }
        unsafe {
            let r = self.readers.get();
            let cur = *r;
            if cur == i32::MAX {
                return false;
            }
            *r = cur + 1;
            true
        }
    }

    /// Try to acquire a write lock. Requires `&mut self` (exclusive).
    pub fn try_write(&mut self, id: u64) -> bool {
        let readers = unsafe { *self.readers.get() };
        if readers != 0 || self.writer != 0 {
            return false;
        }
        self.writer = id;
        true
    }

    /// Release a read lock.
    pub fn release_read(&self) {
        unsafe {
            let r = self.readers.get();
            if *r > 0 {
                *r -= 1;
            }
        }
    }

    /// Release a write lock.
    pub fn release_write(&mut self) {
        self.writer = 0;
    }
}

impl Default for RwState {
    fn default() -> Self {
        Self::new()
    }
}
