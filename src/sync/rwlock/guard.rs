pub struct ReadGuard<'a> {
    state: &'a super::state::RwState,
    released: bool,
}

impl<'a> ReadGuard<'a> {
    pub(crate) fn new(state: &'a super::state::RwState) -> Self {
        Self {
            state,
            released: false,
        }
    }

    pub fn is_released(&self) -> bool {
        self.released
    }
    pub fn reader_count(&self) -> u32 {
        self.state.reader_count()
    }
    pub fn is_write_held(&self) -> bool {
        self.state.is_write()
    }

    pub fn release(mut self) {
        if !self.released {
            self.state.release_read();
            self.released = true;
        }
    }
}

impl<'a> Drop for ReadGuard<'a> {
    fn drop(&mut self) {
        if !self.released {
            self.state.release_read();
            self.released = true;
        }
    }
}

/// Write guard returned by a successful write lock. Must call `release()`.
pub struct WriteGuard<'a> {
    state: &'a mut super::state::RwState,
    id: u64,
    released: bool,
}

impl<'a> WriteGuard<'a> {
    pub(crate) fn new(state: &'a mut super::state::RwState, id: u64) -> Self {
        Self {
            state,
            id,
            released: false,
        }
    }
    /// Writer id.
    pub fn id(&self) -> u64 {
        self.id
    }
    /// Whether released.
    pub fn is_released(&self) -> bool {
        self.released
    }
    /// Whether the lock is held by a writer (it is, by us).
    pub fn is_write_held(&self) -> bool {
        true
    }
    /// Explicitly release the write lock.
    pub fn release(mut self) {
        if !self.released {
            self.state.release_write();
            self.released = true;
        }
    }
}
