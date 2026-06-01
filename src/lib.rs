//! Xin runtime library (`xinr`).
//!
//! Pure runtime support for the Xin language: **XGC** garbage collector and
//! structured concurrency primitives. `no_std` compatible with optional **alloc**.
//! Does **not** provide the Xin standard library.
//!
//! # Example
//!
//! ```ignore
//! use xinr::xgc::Xgc;
//! let mut gc = Xgc::new(1024).unwrap();
//! gc.init().unwrap();
//! ```

#![no_std]
#![allow(clippy::module_name_repetitions, clippy::missing_docs_in_private_items)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
pub use error::*;
pub mod bench;
pub mod sync;
#[cfg(feature = "alloc")]
pub mod xgc;

#[cfg(test)]
mod tests;
