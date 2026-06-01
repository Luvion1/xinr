//! GC trigger: decides when to start a new cycle based on pressure.
//!
//! # Heuristic
//!
//! - Trigger when `meter.allocated >= config.trigger_bytes()`.
//! - Respect `min_interval` between triggers.
//! - Always allow explicit `force()` trigger.

use crate::xgc::pressure::threshold::{PressureConfig, PressureMeter};

/// Trigger state.
#[derive(Debug, Clone, Copy, Default)]
pub struct GcTrigger {
    last_trigger_cycle: u32,
}

impl GcTrigger {
    /// Create a fresh trigger.
    pub const fn new() -> Self {
        Self {
            last_trigger_cycle: 0,
        }
    }

    /// Decide whether to start a new cycle.
    ///
    /// # Returns
    ///
    /// `true` if pressure is high enough OR the caller has not yet seen a
    /// cycle and allocation has happened.
    pub fn should_trigger(
        &self,
        config: &PressureConfig,
        meter: &PressureMeter,
        current_cycle: u32,
    ) -> bool {
        if current_cycle - self.last_trigger_cycle < config.min_interval {
            return false;
        }
        meter.allocated >= config.trigger_bytes()
    }

    /// Mark a trigger as having fired.
    pub fn mark_triggered(&mut self, cycle: u32) {
        self.last_trigger_cycle = cycle;
    }
}
