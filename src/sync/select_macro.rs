//! `select!` macro: ergonomic multi-channel polling.
//!
//! Polls up to 8 channels and runs the first ready arm.
//!
//! # recv! arm
//!
//! Each arm is a `recv(channel)` expression which returns the arm body.
//! First arm to have data wins.
//!
//! ```ignore
//! let result = select! {
//!     0 => recv(a) => idx,
//!     1 => recv(b) => idx,
//! };
//! ```
//!
//! # send! arm (select_send! macro)
//!
//! Each arm is a `send(channel, value)` expression. First arm that can
//! accept wins.
//!
//! ```ignore
//! let r = select_send! {
//!     0 => send(a, 42) => idx,
//!     1 => send(b, 43) => idx,
//! };
//! ```

/// Variant returned by `select_recv` and friends.
pub use super::select::SelectResult;

/// Run the first ready recv arm. Returns the arm body value.
#[macro_export]
macro_rules! select {
    ($($idx:literal => recv($ch:expr) => $arm:expr),* $(,)?) => {{
        $(
            if $ch.try_recv().is_ok() {
                #[allow(unreachable_patterns)]
                Some($arm)
            } else
        )*
        { None }
    }};
}

/// Run the first ready send arm. Returns the arm body value.
#[macro_export]
macro_rules! select_send {
    ($($idx:literal => send($ch:expr, $val:expr) => $arm:expr),* $(,)?) => {{
        $(
            if $ch.try_send($val).is_ok() {
                Some($arm)
            } else
        )*
        { None }
    }};
}
