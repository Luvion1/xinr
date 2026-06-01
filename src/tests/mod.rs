//! Top-level tests for `xinr`. All test modules are gated on `alloc` since
//! the XGC subsystem requires heap allocation.

#[cfg(test)]
mod bench;
#[cfg(test)]
mod sync;
#[cfg(all(test, feature = "alloc"))]
mod xgc;
