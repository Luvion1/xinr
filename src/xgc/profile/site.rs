//! Allocation site identifier.

use core::sync::atomic::{AtomicU32, Ordering};

pub static NEXT_SITE: AtomicU32 = AtomicU32::new(1);

/// Site id (a unique u32 per allocation site).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SiteId(pub u32);

impl SiteId {
    /// The anonymous site id (zero).
    pub const ANON: Self = Self(0);

    /// Allocate a fresh unique id.
    pub fn fresh() -> Self {
        Self(NEXT_SITE.fetch_add(1, Ordering::Relaxed))
    }
}
