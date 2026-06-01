//! Relocation: tracks old→new pointer remapping.

use crate::RuntimeError;
use crate::xgc::colored::ColoredPtr;

const RELOC_CAP: usize = 1024;

/// Statistics from a relocation pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct RelocStats {
    /// Number of objects moved.
    pub moved: u32,
    /// Number skipped (pinned, etc.).
    pub skipped: u32,
}

/// Relocator: records old→new mappings.
pub struct Relocator {
    mappings: [(ColoredPtr, ColoredPtr); RELOC_CAP],
    count: usize,
    active: bool,
}

impl Relocator {
    /// Construct an empty relocator.
    pub const fn new() -> Self {
        Self {
            mappings: [(ColoredPtr::NULL, ColoredPtr::NULL); RELOC_CAP],
            count: 0,
            active: false,
        }
    }

    /// Begin a relocation pass.
    pub fn begin(&mut self) {
        self.active = true;
    }

    /// Record a single move.
    pub fn record_move(&mut self, old: ColoredPtr, new: ColoredPtr) -> Result<(), RuntimeError> {
        if self.count >= RELOC_CAP {
            return Err(RuntimeError::StackOverflow);
        }
        self.mappings[self.count] = (old, new);
        self.count += 1;
        Ok(())
    }

    /// Finish and return statistics.
    pub fn finish(&mut self) -> RelocStats {
        let stats = RelocStats {
            moved: self.count as u32,
            skipped: 0,
        };
        self.count = 0;
        self.active = false;
        stats
    }
}

impl Default for Relocator {
    fn default() -> Self {
        Self::new()
    }
}
