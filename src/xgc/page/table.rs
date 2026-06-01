//! Page table: tracks the state of every page in the heap.

use crate::xgc::page::align::{PAGE_SIZE, REGION_PAGE_COUNT};
use crate::xgc::page::descriptor::{PageDescriptor, PageState};

/// Page table for one region.
pub struct PageTable {
    pages: [PageDescriptor; REGION_PAGE_COUNT],
}

impl PageTable {
    /// Construct a fresh, all-free page table.
    pub const fn new() -> Self {
        Self {
            pages: [PageDescriptor::free(); REGION_PAGE_COUNT],
        }
    }

    /// Fetch a page descriptor.
    pub fn get(&self, page_idx: usize) -> Option<&PageDescriptor> {
        self.pages.get(page_idx)
    }

    /// Mark a byte range as used.
    pub fn mark_range_used(&mut self, byte_offset: usize, bytes: usize) {
        let start_page = byte_offset / PAGE_SIZE;
        let end_byte = byte_offset.saturating_add(bytes);
        let end_page = end_byte.div_ceil(PAGE_SIZE);
        for i in start_page..end_page.min(self.pages.len()) {
            self.pages[i].mark_used();
        }
    }

    /// Mark a byte range as dirty.
    pub fn mark_range_dirty(&mut self, byte_offset: usize, bytes: usize) {
        let start_page = byte_offset / PAGE_SIZE;
        let end_byte = byte_offset.saturating_add(bytes);
        let end_page = end_byte.div_ceil(PAGE_SIZE);
        for i in start_page..end_page.min(self.pages.len()) {
            self.pages[i].mark_dirty();
        }
    }

    /// Reset all pages to free.
    pub fn clear(&mut self) {
        for p in self.pages.iter_mut() {
            p.mark_free();
        }
    }

    /// Number of used pages.
    pub fn used_count(&self) -> usize {
        self.pages.iter().filter(|p| p.is_used()).count()
    }

    /// Total pages.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Visit each page in the given state.
    pub fn for_each_in<F: FnMut(usize, &PageDescriptor)>(&self, state: PageState, mut f: F) {
        for (i, p) in self.pages.iter().enumerate() {
            if p.state == state {
                f(i, p);
            }
        }
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}
