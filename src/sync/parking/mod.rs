//! Parking lot: token-based park/unpark registry for blocking operations.

pub mod lot;
pub mod permit;

pub use lot::{ParkedThread, ParkingLot};
pub use permit::Permit;
