//! Fiber stack: a virtual stack region (size, base, top).
//!
//! A fiber's stack is a `[u8; N]` region embedded in the fiber struct, so
//! no allocation is required. The actual address is reported via `base()`
//! and `top()`. In a real runtime these would point into mmap'd memory.

/// Standard page size (4 KiB). Re-exposed here so the fiber module does
/// not depend on the `alloc`-gated xgc module.
pub const PAGE_SIZE: usize = 4096;

/// Default stack size: 4 KiB (one page). Tunable per-runtime.
pub const DEFAULT_STACK_SIZE: usize = 4 * 1024;

/// A fiber's stack region.
pub struct FiberStack {
    bytes: [u8; DEFAULT_STACK_SIZE],
    top: *mut u8,
    bottom: *mut u8,
}

impl FiberStack {
    /// Construct a fresh, unused stack.
    pub const fn new() -> Self {
        Self {
            bytes: [0; DEFAULT_STACK_SIZE],
            top: core::ptr::null_mut(),
            bottom: core::ptr::null_mut(),
        }
    }

    /// Initialize the stack boundaries. Call once after construction.
    pub fn init(&mut self) {
        let base = self.bytes.as_mut_ptr();
        // Stacks grow downward, so top = base + size.
        unsafe {
            self.top = base.add(self.bytes.len());
            self.bottom = base;
        }
    }

    /// Top of the stack (highest address).
    pub fn top(&self) -> *mut u8 {
        self.top
    }

    /// Bottom of the stack (lowest address).
    pub fn bottom(&self) -> *mut u8 {
        self.bottom
    }

    /// Stack size in bytes.
    pub const fn size(&self) -> usize {
        DEFAULT_STACK_SIZE
    }

    /// Bytes remaining until guard page (here: just stack size).
    pub fn remaining(&self) -> usize {
        if self.top.is_null() || self.bottom.is_null() {
            return 0;
        }
        unsafe { self.top.offset_from(self.bottom) as usize }
    }

    /// Number of 4 KiB pages that fit in this stack.
    pub fn pages(&self) -> usize {
        DEFAULT_STACK_SIZE / PAGE_SIZE
    }
}

impl Default for FiberStack {
    fn default() -> Self {
        Self::new()
    }
}
