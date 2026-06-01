//! Colored pointers: ZGC-style pointer + metadata tag in unused bits.
//!
//! A `ColoredPtr` packs an address with a 2-bit `Color` tag and a single
//! `relocated` flag. On x86-64 with 48-bit addressing, the top 16 bits of
//! any user pointer are unused, so we have plenty of room to spare.

use core::fmt;

/// Object liveness color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Not yet reached.
    White,
    /// Reached, children not yet visited.
    Grey,
    /// Reached, all children visited.
    Black,
}

impl Color {
    /// Display label.
    pub fn label(self) -> &'static str {
        match self {
            Color::White => "white",
            Color::Grey => "grey",
            Color::Black => "black",
        }
    }
}

/// 64-bit colored pointer: low 2 bits = color, bit 2 = relocated.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredPtr(pub u64);

impl ColoredPtr {
    /// Sentinel null pointer.
    pub const NULL: Self = Self(0);

    /// Wrap a raw address with a color.
    pub const fn new(addr: usize, color: Color) -> Self {
        let color_bits = match color {
            Color::White => 0,
            Color::Grey => 1,
            Color::Black => 2,
        };
        Self((addr as u64) | color_bits)
    }

    /// Raw address.
    pub fn addr(self) -> usize {
        (self.0 & !0b111) as usize
    }

    /// Color tag.
    pub fn color(self) -> Color {
        match self.0 & 0b11 {
            0 => Color::White,
            1 => Color::Grey,
            _ => Color::Black,
        }
    }

    /// Whether the object has been relocated.
    pub fn is_relocated(self) -> bool {
        (self.0 & 0b100) != 0
    }

    /// Mark as relocated.
    pub fn mark_relocated(&mut self) {
        self.0 |= 0b100;
    }
}

impl fmt::Debug for ColoredPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ColoredPtr({:#x}, {})",
            self.addr(),
            self.color().label()
        )
    }
}
