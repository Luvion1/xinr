//! Pin handle: marker that prevents the GC from moving the underlying object.
//!
//! Pinned objects are still scanned for references (mark phase) but are
//! skipped during relocation. Useful for FFI handles, memory-mapped I/O
//! buffers, and OS resource wrappers.

use crate::xgc::colored::ColoredPtr;

/// Opaque pin handle. Cloning increments the pin count; dropping decrements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinHandle {
    id: u32,
}

impl PinHandle {
    /// Sentinel for an invalid handle.
    pub const INVALID: PinHandle = PinHandle { id: 0 };

    /// Whether this handle is valid.
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }

    /// Internal numeric id.
    pub fn id(self) -> u32 {
        self.id
    }
}

/// Build a PinHandle from a raw id. Internal use only.
pub fn handle_from_id(id: u32) -> PinHandle {
    PinHandle { id }
}

/// Reference to a pinned object.
#[derive(Debug, Clone, Copy)]
pub struct PinnedObject {
    pub ptr: ColoredPtr,
    pub handle: PinHandle,
}

impl PinnedObject {
    /// Construct a pinned object reference.
    pub const fn new(ptr: ColoredPtr, handle: PinHandle) -> Self {
        Self { ptr, handle }
    }
}
