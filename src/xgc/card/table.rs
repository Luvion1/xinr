//! Card table: array of atomic card bytes covering one region.

use crate::xgc::card::byte::{CARDS_PER_REGION, CardByte, CardState};

/// Card table for one region. Aligned to a cache line to avoid false sharing
/// when mutator threads write to adjacent cards.
#[repr(align(64))]
pub struct CardTable {
    cards: [CardByte; CARDS_PER_REGION],
}

impl CardTable {
    /// Construct a clean card table.
    pub const fn new() -> Self {
        Self {
            cards: [const { CardByte::new() }; CARDS_PER_REGION],
        }
    }

    /// Mark the card containing the given offset within the region dirty.
    pub fn mark_dirty_at(&self, byte_offset: usize) {
        if byte_offset >= crate::xgc::region::REGION_SIZE {
            return;
        }
        let idx = byte_offset / crate::xgc::card::byte::CARD_SIZE;
        if idx < self.cards.len() {
            self.cards[idx].mark_dirty();
        }
    }

    /// Test the card at `byte_offset`.
    pub fn is_dirty_at(&self, byte_offset: usize) -> bool {
        if byte_offset >= crate::xgc::region::REGION_SIZE {
            return false;
        }
        let idx = byte_offset / crate::xgc::card::byte::CARD_SIZE;
        if idx < self.cards.len() {
            self.cards[idx].load() == CardState::Dirty
        } else {
            false
        }
    }

    /// Iterate all dirty card indices in this table, calling `f(idx)` for each.
    pub fn for_each_dirty<F: FnMut(usize)>(&self, mut f: F) {
        for (i, c) in self.cards.iter().enumerate() {
            if c.load() == CardState::Dirty {
                f(i);
            }
        }
    }

    /// Reset all cards to clean.
    pub fn clear(&self) {
        for c in self.cards.iter() {
            c.store(CardState::Clean);
        }
    }

    /// Number of cards.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Whether the table has zero cards.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for CardTable {
    fn default() -> Self {
        Self::new()
    }
}
