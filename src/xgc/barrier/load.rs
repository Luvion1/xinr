//! Load barrier: enforces happens-before edge on reads of colored pointers.

use crate::xgc::colored::ColoredPtr;

/// Barrier trait for pointer reads.
pub trait LoadBarrier {
    /// Read pointer with barrier.
    fn load(&self) -> ColoredPtr;
}

impl LoadBarrier for ColoredPtr {
    fn load(&self) -> ColoredPtr {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::Acquire);
        *self
    }
}
