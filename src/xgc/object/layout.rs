//! Object size class and pointer alignment.

/// Object size class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    /// 32-byte object.
    S32,
    /// 64-byte object.
    S64,
    /// 128-byte object.
    S128,
    /// 256-byte object.
    S256,
    /// 512-byte object.
    S512,
    /// 2 KiB object.
    S2K,
    /// Larger than 2 KiB.
    Large,
}

impl SizeClass {
    /// Pick a size class for a payload size.
    pub fn for_payload(bytes: usize) -> SizeClass {
        match bytes {
            0..=32 => SizeClass::S32,
            33..=64 => SizeClass::S64,
            65..=128 => SizeClass::S128,
            129..=256 => SizeClass::S256,
            257..=512 => SizeClass::S512,
            513..=2048 => SizeClass::S2K,
            _ => SizeClass::Large,
        }
    }
}

/// Pointer alignment requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// 8-byte aligned.
    A8,
    /// 16-byte aligned.
    A16,
    /// 64-byte aligned (cache line).
    A64,
}

impl Alignment {
    /// Alignment in bytes.
    pub const fn bytes(self) -> usize {
        match self {
            Alignment::A8 => 8,
            Alignment::A16 => 16,
            Alignment::A64 => 64,
        }
    }
}
