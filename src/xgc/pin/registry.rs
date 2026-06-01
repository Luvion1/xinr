//! Pin registry: tracks all currently pinned objects.
//!
//! The GC consults this registry during relocation to skip pinned objects.
//! Pin counts are kept so a single object can be pinned multiple times.
//!
//! The handle id is the slot index + 1 (0 = INVALID). This keeps the lookup
//! O(1) without a side table.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;
use crate::xgc::pin::handle::{PinHandle, handle_from_id};

const REGISTRY_CAP: usize = 256;

/// One entry in the pin registry.
#[derive(Debug, Clone, Copy)]
pub struct PinEntry {
    pub ptr: ColoredPtr,
    pub count: u32,
}

/// Fixed-capacity registry of pinned objects.
pub struct PinRegistry {
    entries: [PinEntry; REGISTRY_CAP],
    next_slot: usize,
}

impl PinRegistry {
    /// Construct an empty registry.
    pub const fn new() -> Self {
        Self {
            entries: [PinEntry {
                ptr: ColoredPtr::new(0, crate::xgc::colored::Color::White),
                count: 0,
            }; REGISTRY_CAP],
            next_slot: 0,
        }
    }

    /// Pin an object. Returns a handle.
    ///
    /// # Errors
    ///
    /// Returns `OutOfMemory` if the registry is full.
    pub fn pin(&mut self, ptr: ColoredPtr) -> Result<PinHandle, RuntimeError> {
        // If already pinned, increment count.
        for e in self.entries.iter_mut() {
            if e.count > 0 && e.ptr == ptr {
                e.count += 1;
                let slot = (e as *const _ as usize - self.entries.as_ptr() as usize)
                    / core::mem::size_of::<PinEntry>();
                return Ok(handle_from_id((slot as u32) + 1));
            }
        }
        // Otherwise, allocate a new entry.
        for _ in 0..REGISTRY_CAP {
            let slot = self.next_slot % REGISTRY_CAP;
            self.next_slot = (self.next_slot + 1) % REGISTRY_CAP;
            let entry = &mut self.entries[slot];
            if entry.count == 0 {
                entry.ptr = ptr;
                entry.count = 1;
                return Ok(handle_from_id((slot as u32) + 1));
            }
        }
        Err(RuntimeError::OutOfMemory)
    }

    /// Unpin an object. Returns true if the entry was fully released.
    pub fn unpin(&mut self, handle: PinHandle) -> bool {
        if !handle.is_valid() {
            return false;
        }
        let slot = (handle.id() as usize).saturating_sub(1);
        if slot >= REGISTRY_CAP {
            return false;
        }
        let entry = &mut self.entries[slot];
        if entry.count == 0 {
            return false;
        }
        entry.count -= 1;
        if entry.count == 0 {
            entry.ptr = ColoredPtr::new(0, crate::xgc::colored::Color::White);
            return true;
        }
        false
    }

    /// Whether the given pointer is currently pinned.
    pub fn is_pinned(&self, ptr: ColoredPtr) -> bool {
        self.entries.iter().any(|e| e.count > 0 && e.ptr == ptr)
    }

    /// Number of currently pinned objects.
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.count > 0).count()
    }

    /// Whether the registry has no pinned objects.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for PinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
