//! Heap pressure subsystem: threshold tracking and GC triggers.

pub mod threshold;
pub mod trigger;

pub use threshold::{PressureConfig, PressureMeter};
pub use trigger::GcTrigger;
