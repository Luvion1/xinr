//! Object header: 12-byte metadata stored before each allocated object.

use crate::xgc::colored::Color;
use crate::xgc::object::layout::{Alignment, SizeClass};

/// Magic value stored in every valid header.
pub const HEADER_MAGIC: u32 = 0xDE_C0_ED;

/// Per-object header (12 bytes).
#[derive(Debug, Clone, Copy)]
pub struct ObjectHeader {
    /// Magic + flags.
    pub magic: u32,
    /// Object id (hash of allocation site or runtime-assigned).
    pub id: u32,
    /// Object size class.
    pub size_class: SizeClass,
    /// Pointer alignment.
    pub align: Alignment,
    /// GC color.
    color: Color,
}

impl ObjectHeader {
    /// Construct a fresh header.
    pub const fn new(id: u32, size_class: SizeClass, align: Alignment) -> Self {
        Self {
            magic: HEADER_MAGIC,
            id,
            size_class,
            align,
            color: Color::White,
        }
    }

    /// Whether the magic is valid.
    pub fn is_valid(&self) -> bool {
        self.magic == HEADER_MAGIC
    }

    /// Read color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Set color.
    pub fn set_color(&mut self, c: Color) {
        self.color = c;
    }
}
