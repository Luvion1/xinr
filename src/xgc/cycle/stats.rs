//! Cycle detector statistics.

use crate::xgc::colored::ColoredPtr;

/// One candidate for cycle detection.
#[derive(Debug, Clone, Copy)]
pub struct CycleCandidate {
    /// The object under test.
    pub object: ColoredPtr,
    /// Associated weak ref id.
    pub weak_ref: u32,
}

/// Aggregate statistics from a cycle detection pass.
#[derive(Debug, Clone, Copy)]
pub struct CycleStats {
    /// Number of candidates seen.
    pub candidates: u32,
    /// Number actually reclaimed.
    pub reclaimed: u32,
}

impl CycleStats {
    /// Construct zeroed stats.
    pub const fn new() -> Self {
        Self {
            candidates: 0,
            reclaimed: 0,
        }
    }

    /// Hit rate in `[0.0, 1.0]`. Returns 0.0 if no candidates.
    pub fn hit_rate(&self) -> f32 {
        if self.candidates == 0 {
            0.0
        } else {
            self.reclaimed as f32 / self.candidates as f32
        }
    }
}

impl Default for CycleStats {
    fn default() -> Self {
        Self::new()
    }
}
