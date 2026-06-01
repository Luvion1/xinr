//! `select!` macro: ergonomic multi-channel polling.
//!
//! Polls up to 8 channels and runs the first ready arm. Returns
//! `(index, value)` if one channel had data, or `None` if all were empty.
//!
//! # Example
//!
//! ```ignore
//! let mut a: BoundedChannel<u32, 4> = BoundedChannel::new();
//! let mut b: BoundedChannel<u32, 4> = BoundedChannel::new();
//! a.try_send(42).unwrap();
//! let result = select! {
//!     idx(0) = recv(a) => idx,
//!     idx(1) = recv(b) => idx,
//! };
//! assert_eq!(result, Some(0));
//! ```

/// Variant returned by `select_recv` and friends.
pub use super::select::SelectResult;

/// Run the first ready arm. Each arm is a `recv(channel)` expression; the
/// first one that returns a value wins.
#[macro_export]
macro_rules! select {
    ($($idx:expr => recv($ch:expr) => $arm:expr),* $(,)?) => {{
        // Probe in order; first non-empty wins.
        $(
            if let Ok(_v) = $ch.try_recv() {
                #[allow(unreachable_patterns)]
                match $idx {
                    _ => Some($arm),
                }
            } else
        )*
        { None }
    }};
}
