//! Single hazard slot.

use core::sync::atomic::{AtomicPtr, Ordering};

/// One hazard pointer slot.
pub struct HazardSlot {
    ptr: AtomicPtr<u8>,
    active: bool,
}

impl HazardSlot {
    /// Construct an inactive slot.
    pub const fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(core::ptr::null_mut()),
            active: false,
        }
    }

    /// Whether the slot is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Read the protected pointer.
    pub fn get(&self) -> *mut u8 {
        self.ptr.load(Ordering::Acquire)
    }

    /// Publish a protected pointer.
    pub fn publish<T>(&mut self, p: *mut T) {
        self.ptr.store(p as *mut u8, Ordering::Release);
        self.active = true;
    }

    /// Clear the protected pointer.
    pub fn clear(&mut self) {
        self.ptr.store(core::ptr::null_mut(), Ordering::Release);
        self.active = false;
    }
}

impl Default for HazardSlot {
    fn default() -> Self {
        Self::new()
    }
}
