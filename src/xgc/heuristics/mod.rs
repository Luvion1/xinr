//! Adaptive heuristics: collection triggers and heap sizing.

pub mod sizing;
pub mod trigger;

pub use sizing::HeapSizing;
pub use trigger::{CollectionTrigger, Trigger};
