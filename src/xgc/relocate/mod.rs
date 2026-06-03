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

    /// Forward a pointer through the relocation map. If `ptr` was relocated,
    /// returns the new location; otherwise returns `ptr` unchanged.
    pub fn forward(&self, ptr: ColoredPtr) -> ColoredPtr {
        if !self.active {
            return ptr;
        }
        for i in 0..self.count {
            let (old, new) = self.mappings[i];
            if old.addr() == ptr.addr() {
                return new;
            }
        }
        ptr
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xgc::Color;

    #[test]
    fn forward_returns_unchanged_when_inactive() {
        let reloc = Relocator::new();
        let ptr = ColoredPtr::new(0x1000, Color::White);
        let result = reloc.forward(ptr);
        assert_eq!(result, ptr);
    }

    #[test]
    fn forward_returns_new_when_mapped() {
        let mut reloc = Relocator::new();
        reloc.begin();
        let old = ColoredPtr::new(0x1000, Color::White);
        let new = ColoredPtr::new(0x2000, Color::Black);
        reloc.record_move(old, new).unwrap();
        let result = reloc.forward(old);
        assert_eq!(result.addr(), 0x2000);
    }

    #[test]
    fn forward_ignores_unmapped_ptr() {
        let mut reloc = Relocator::new();
        reloc.begin();
        let moved = ColoredPtr::new(0x1000, Color::White);
        let new = ColoredPtr::new(0x2000, Color::Black);
        reloc.record_move(moved, new).unwrap();

        let other = ColoredPtr::new(0x9999, Color::White);
        assert_eq!(reloc.forward(other), other);
    }
}
