//! Parking permit: a token issued to a thread that allows it to park.
//!
//! A permit is acquired before park and released after unpark. A thread
//! can only park if it holds a permit; the permit is consumed on park
//! and a new one is issued by the unparker.

/// Parking permit. Opaque, token-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Permit(pub u64);

impl Permit {
    /// Sentinel for "no permit".
    pub const NONE: Permit = Permit(0);
    /// Whether this permit is valid (non-zero token).
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}
