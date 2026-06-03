//! Card table tests.

use crate::xgc::card::byte::{CardByte, CardState};

#[test]
fn card_byte_is_clean_initially() {
    let card = CardByte::new();
    assert_eq!(card.load(), CardState::Clean);
}

#[test]
fn card_byte_can_be_marked_dirty() {
    let card = CardByte::new();
    card.mark_dirty();
    assert_eq!(card.load(), CardState::Dirty);
}