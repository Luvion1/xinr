//! Mark phase state machine: Idle → Marking → Relocating → Idle.

use core::sync::atomic::{AtomicU8, Ordering};

/// Phase of the GC state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkPhase {
    /// GC idle.
    Idle = 0,
    /// Mark cycle in progress.
    Marking = 1,
    /// Relocate cycle in progress.
    Relocating = 2,
}

impl MarkPhase {
    /// Display label.
    pub fn as_str(self) -> &'static str {
        match self {
            MarkPhase::Idle => "idle",
            MarkPhase::Marking => "marking",
            MarkPhase::Relocating => "relocating",
        }
    }
}

/// Atomic phase cell.
pub struct PhaseCell {
    inner: AtomicU8,
}

impl PhaseCell {
    /// Construct a new cell in `Idle`.
    pub const fn new() -> Self {
        Self {
            inner: AtomicU8::new(0),
        }
    }

    /// Load current phase.
    pub fn load(&self) -> MarkPhase {
        match self.inner.load(Ordering::Acquire) {
            1 => MarkPhase::Marking,
            2 => MarkPhase::Relocating,
            _ => MarkPhase::Idle,
        }
    }

    /// Store a new phase.
    pub fn store(&self, p: MarkPhase) {
        self.inner.store(p as u8, Ordering::Release);
    }

    /// Compare-and-swap. Returns `true` on success.
    pub fn cas(&self, expected: MarkPhase, next: MarkPhase) -> bool {
        self.inner
            .compare_exchange(
                expected as u8,
                next as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}

impl Default for PhaseCell {
    fn default() -> Self {
        Self::new()
    }
}
