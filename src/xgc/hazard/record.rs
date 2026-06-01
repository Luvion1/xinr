//! Multi-slot hazard record.

use crate::xgc::hazard::slot::HazardSlot;

const MAX_SLOTS: usize = 16;

/// Record of multiple hazard slots.
#[allow(dead_code)]
pub struct HazardRecord {
    slots: [HazardSlot; MAX_SLOTS],
    count: usize,
}

impl HazardRecord {
    /// Construct an empty record.
    pub const fn new() -> Self {
        Self {
            slots: [const { HazardSlot::new() }; MAX_SLOTS],
            count: MAX_SLOTS,
        }
    }

    /// Whether any slot is active.
    pub fn any_active(&self) -> bool {
        self.slots.iter().any(|s| s.is_active())
    }

    /// Whether the slot at `idx` is active.
    pub fn is_active(&self, idx: usize) -> bool {
        idx < self.slots.len() && self.slots[idx].is_active()
    }

    /// Publish to slot `idx`.
    pub fn publish<T>(&mut self, idx: usize, p: *mut T) {
        if idx < self.slots.len() {
            self.slots[idx].publish(p);
        }
    }

    /// Clear slot `idx`.
    pub fn clear(&mut self, idx: usize) {
        if idx < self.slots.len() {
            self.slots[idx].clear();
        }
    }

    /// Clear all slots.
    pub fn clear_all(&mut self) {
        for s in self.slots.iter_mut() {
            s.clear();
        }
    }
}

impl Default for HazardRecord {
    fn default() -> Self {
        Self::new()
    }
}
