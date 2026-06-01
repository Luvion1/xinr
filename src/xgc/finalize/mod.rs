//! Finalization and weak references.

pub mod queue;
pub mod weak;

pub use queue::{FinalizationQueue, FinalizeEntry, FinalizerId, QUEUE_CAP};
pub use weak::{WEAK_TABLE_CAP, WeakEntry, WeakRef, WeakTable, try_upgrade};
