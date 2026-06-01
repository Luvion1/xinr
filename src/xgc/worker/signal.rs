//! GC signal: atomic flag for waking the GC worker thread.
//!
//! The worker sleeps in a low-power wait. The mutator thread sets the signal
//! to request a cycle. After processing, the worker clears the signal and
//! goes back to sleep.

use portable_atomic::{AtomicBool, AtomicU32, Ordering};

/// Work order: what the worker should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkOrder {
    /// No work pending; worker may sleep.
    None = 0,
    /// Run a mark cycle.
    Mark = 1,
    /// Run a sweep cycle.
    Sweep = 2,
    /// Run a relocation cycle.
    Relocate = 3,
    /// Shutdown the worker.
    Shutdown = 4,
}

impl WorkOrder {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => Self::Mark,
            2 => Self::Sweep,
            3 => Self::Relocate,
            4 => Self::Shutdown,
            _ => Self::None,
        }
    }
}

/// Signal cell: combination of `work_order` and a wake-up flag.
#[repr(align(64))]
pub struct GcSignal {
    order: AtomicU32,
    wake: AtomicBool,
}

impl GcSignal {
    /// Construct a fresh signal in None state.
    pub const fn new() -> Self {
        Self {
            order: AtomicU32::new(WorkOrder::None as u32),
            wake: AtomicBool::new(false),
        }
    }

    /// Current work order.
    pub fn order(&self) -> WorkOrder {
        WorkOrder::from_byte(self.order.load(Ordering::Acquire) as u8)
    }

    /// Whether the worker is requested to wake.
    pub fn is_wake(&self) -> bool {
        self.wake.load(Ordering::Acquire)
    }

    /// Request a specific work order. Sets wake flag.
    pub fn request(&self, o: WorkOrder) {
        self.order.store(o as u32, Ordering::Release);
        self.wake.store(true, Ordering::Release);
    }

    /// Consume the wake signal. Returns the order.
    pub fn consume(&self) -> WorkOrder {
        let o = self.order();
        self.wake.store(false, Ordering::Release);
        o
    }
}

impl Default for GcSignal {
    fn default() -> Self {
        Self::new()
    }
}
