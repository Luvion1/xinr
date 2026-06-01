//! Page alignment helpers.

use crate::xgc::region::REGION_SIZE;

/// Bytes per page.
pub const PAGE_SIZE: usize = 4096;

/// Pages per region.
pub const REGION_PAGE_COUNT: usize = REGION_SIZE / PAGE_SIZE;

/// Round `addr` up to the next page boundary.
pub const fn page_round_up(addr: usize) -> usize {
    addr.div_ceil(PAGE_SIZE) * PAGE_SIZE
}

/// Round `addr` down to the previous page boundary.
pub const fn page_round_down(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

/// Whether `addr` is page-aligned.
pub const fn is_page_aligned(addr: usize) -> bool {
    addr.is_multiple_of(PAGE_SIZE)
}
