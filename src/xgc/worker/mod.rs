//! Concurrent mark worker: coordination between mutator threads and the
//! background GC thread. The worker is signaled via `GcSignal`; actual
//! thread spawning is deferred to a platform-specific runtime.

pub mod signal;
pub mod thread;

pub use signal::{GcSignal, WorkOrder};
pub use thread::{GcWorker, WorkerState};
