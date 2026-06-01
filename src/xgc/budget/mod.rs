//! Cycle budget: time limit per GC phase.
//!
//! The GC checks the budget at safe points during each phase. If the
//! budget is exceeded, the GC yields and resumes on the next cycle.

pub mod clock;

use crate::xgc::budget::clock::Instant;

/// Per-phase budget configuration (in milliseconds).
#[derive(Debug, Clone, Copy)]
pub struct CycleBudget {
    /// Maximum time for the mark phase.
    pub mark_ms: u64,
    /// Maximum time for the sweep phase.
    pub sweep_ms: u64,
    /// Maximum time for the relocation phase.
    pub relocate_ms: u64,
    /// Maximum time for the remap phase.
    pub remap_ms: u64,
}

impl CycleBudget {
    /// Default budgets: 5 ms per phase (very conservative for soft real-time).
    pub const fn default_soft() -> Self {
        Self {
            mark_ms: 5,
            sweep_ms: 2,
            relocate_ms: 5,
            remap_ms: 1,
        }
    }

    /// Generous budgets for non-real-time use.
    pub const fn default_generous() -> Self {
        Self {
            mark_ms: 50,
            sweep_ms: 20,
            relocate_ms: 50,
            remap_ms: 10,
        }
    }

    /// No budgets (unbounded).
    pub const fn unbounded() -> Self {
        Self {
            mark_ms: u64::MAX,
            sweep_ms: u64::MAX,
            relocate_ms: u64::MAX,
            remap_ms: u64::MAX,
        }
    }
}

/// Cycle budget tracker: records start time and checks expiration.
pub struct BudgetTracker {
    budget: CycleBudget,
    start: Instant,
    elapsed: u64,
}

impl BudgetTracker {
    /// Begin a new budget window starting at `start`.
    pub const fn new(budget: CycleBudget, start: Instant) -> Self {
        Self {
            budget,
            start,
            elapsed: 0,
        }
    }

    /// Reset the start instant.
    pub fn reset(&mut self, start: Instant) {
        self.start = start;
        self.elapsed = 0;
    }

    /// Update elapsed time.
    pub fn tick(&mut self, now: Instant) {
        self.elapsed = self.start.elapsed_ms(now);
    }

    /// Current elapsed ms.
    pub fn elapsed(&self) -> u64 {
        self.elapsed
    }

    /// Whether the mark budget is exceeded.
    pub fn mark_exceeded(&self) -> bool {
        self.elapsed > self.budget.mark_ms
    }

    /// Whether the sweep budget is exceeded.
    pub fn sweep_exceeded(&self) -> bool {
        self.elapsed > self.budget.sweep_ms
    }

    /// Whether the relocate budget is exceeded.
    pub fn relocate_exceeded(&self) -> bool {
        self.elapsed > self.budget.relocate_ms
    }

    /// Whether the remap budget is exceeded.
    pub fn remap_exceeded(&self) -> bool {
        self.elapsed > self.budget.remap_ms
    }
}
