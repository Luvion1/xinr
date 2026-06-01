//! Card table: coarse-grained dirty tracking for write barriers.
//!
//! A card is a fixed-size chunk of the heap (default 512 bytes). When a
//! mutator writes a pointer into a field, the corresponding card is marked
//! dirty. During the mark phase, the GC only scans dirty cards.

use crate::xgc::region::REGION_SIZE;
use core::sync::atomic::{AtomicU8, Ordering};

/// Size of one card in bytes.
pub const CARD_SIZE: usize = 512;

/// Number of cards per region.
pub const CARDS_PER_REGION: usize = REGION_SIZE / CARD_SIZE;

/// Card state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardState {
    /// Card is clean; no need to scan.
    Clean = 0,
    /// Card has at least one dirty slot; must be scanned.
    Dirty = 1,
}

impl CardState {
    pub fn from_byte(b: u8) -> Self {
        if b == 0 { Self::Clean } else { Self::Dirty }
    }
}

/// Atomic card byte.
#[repr(align(64))]
pub struct CardByte(pub AtomicU8);

impl CardByte {
    pub const fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    pub fn load(&self) -> CardState {
        CardState::from_byte(self.0.load(Ordering::Acquire))
    }

    pub fn store(&self, s: CardState) {
        self.0.store(s as u8, Ordering::Release);
    }

    /// Atomically mark as dirty.
    pub fn mark_dirty(&self) {
        self.0.store(CardState::Dirty as u8, Ordering::Release);
    }
}

impl Default for CardByte {
    fn default() -> Self {
        Self::new()
    }
}
