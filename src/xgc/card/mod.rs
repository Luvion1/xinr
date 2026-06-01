//! Card table: coarse-grained dirty tracking for write barriers.

pub mod byte;
pub mod table;

pub use byte::{CARD_SIZE, CARDS_PER_REGION, CardByte, CardState};
pub use table::CardTable;
