//! Worker thread: skeleton for the background GC thread.
//!
//! This iteration provides the coordination surface (signal, work order
//! queue) but does not spawn an OS thread. The actual spawn is a future
//! iteration tied to platform-specific runtime (e.g. `std::thread`).

use crate::xgc::Xgc;
use crate::xgc::worker::signal::{GcSignal, WorkOrder};
use core::sync::atomic::AtomicBool;

/// State of a worker thread (whether running).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkerState {
    /// Worker is sleeping.
    Idle = 0,
    /// Worker is processing.
    Busy = 1,
    /// Worker has shut down.
    Stopped = 2,
}

/// Handle to one worker. Owned by the runtime.
#[allow(dead_code)]
pub struct GcWorker {
    pub signal: GcSignal,
    state: AtomicBool,
}

impl GcWorker {
    /// Construct a fresh worker.
    pub const fn new() -> Self {
        Self {
            signal: GcSignal::new(),
            state: AtomicBool::new(false),
        }
    }

    /// Request a mark cycle. Non-blocking; the actual cycle runs when the
    /// worker thread polls.
    pub fn request_mark(&self) {
        self.signal.request(WorkOrder::Mark);
    }

    /// Request a relocation cycle.
    pub fn request_relocate(&self) {
        self.signal.request(WorkOrder::Relocate);
    }

    /// Request a shutdown.
    pub fn request_shutdown(&self) {
        self.signal.request(WorkOrder::Shutdown);
    }

    /// Process one work order synchronously. Returns the order that was
    /// processed. Use this in single-threaded or test scenarios.
    pub fn process_one(&self, gc: &mut Xgc) -> WorkOrder {
        let order = self.signal.consume();
        match order {
            WorkOrder::None => WorkOrder::None,
            WorkOrder::Mark => {
                let _ = gc.begin_mark();
                WorkOrder::Mark
            }
            WorkOrder::Sweep => WorkOrder::Sweep,
            WorkOrder::Relocate => {
                gc.begin_relocate();
                WorkOrder::Relocate
            }
            WorkOrder::Shutdown => {
                let _ = gc.shutdown();
                WorkOrder::Shutdown
            }
        }
    }
}

impl Default for GcWorker {
    fn default() -> Self {
        Self::new()
    }
}
