//! Per-page metadata.

/// Page state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    /// Available for allocation.
    Free,
    /// Has at least one live object.
    Used,
    /// Reserved by the system.
    Reserved,
    /// Has been written since last GC.
    Dirty,
}

impl PageState {
    /// Convert a raw byte to a `PageState` (unknown → Free).
    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => PageState::Free,
            1 => PageState::Used,
            2 => PageState::Reserved,
            3 => PageState::Dirty,
            _ => PageState::Free,
        }
    }
}

/// Per-page state and metadata.
#[derive(Debug, Clone, Copy)]
pub struct PageDescriptor {
    pub state: PageState,
    pub pin_count: u8,
}

impl PageDescriptor {
    /// Construct a free page.
    pub const fn free() -> Self {
        Self {
            state: PageState::Free,
            pin_count: 0,
        }
    }

    /// Mark this page as used.
    pub fn mark_used(&mut self) {
        self.state = PageState::Used;
    }

    /// Mark this page as dirty.
    pub fn mark_dirty(&mut self) {
        self.state = PageState::Dirty;
    }

    /// Mark this page as free.
    pub fn mark_free(&mut self) {
        self.state = PageState::Free;
    }

    /// Whether the page is free.
    pub fn is_free(&self) -> bool {
        self.state == PageState::Free
    }

    /// Whether the page is used (Used or Dirty both count).
    pub fn is_used(&self) -> bool {
        matches!(self.state, PageState::Used | PageState::Dirty)
    }
}
