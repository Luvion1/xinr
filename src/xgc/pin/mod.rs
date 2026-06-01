//! Object pinning: prevent GC relocation for FFI handles and OS resources.

pub mod handle;
pub mod registry;

pub use handle::{PinHandle, PinnedObject, handle_from_id};
pub use registry::{PinEntry, PinRegistry};
