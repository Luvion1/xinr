//! Single hazard slot.

use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

pub struct HazardSlot {
    ptr: AtomicPtr<u8>,
    active: AtomicBool,
}

impl HazardSlot {
    pub const fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(core::ptr::null_mut()),
            active: AtomicBool::new(false),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn get(&self) -> *mut u8 {
        self.ptr.load(Ordering::Acquire)
    }

    pub fn publish<T>(&mut self, p: *mut T) {
        self.ptr.store(p as *mut u8, Ordering::Release);
        self.active.store(true, Ordering::Release);
    }

    pub fn clear(&mut self) {
        self.ptr.store(core::ptr::null_mut(), Ordering::Release);
        self.active.store(false, Ordering::Release);
    }
}

impl Default for HazardSlot {
    fn default() -> Self {
        Self::new()
    }
}
