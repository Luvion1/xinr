//! Slab size: power-of-two fixed allocation size.

/// A slab size descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlabSize(pub u32);

impl SlabSize {
    /// Smallest slab (16 bytes payload).
    pub const S16: SlabSize = SlabSize(16);
    /// 32-byte slab.
    pub const S32: SlabSize = SlabSize(32);
    /// 64-byte slab.
    pub const S64: SlabSize = SlabSize(64);
    /// 128-byte slab.
    pub const S128: SlabSize = SlabSize(128);
    /// 256-byte slab.
    pub const S256: SlabSize = SlabSize(256);
    /// 512-byte slab.
    pub const S512: SlabSize = SlabSize(512);

    /// All standard sizes.
    pub const ALL: [SlabSize; 6] = [
        SlabSize::S16,
        SlabSize::S32,
        SlabSize::S64,
        SlabSize::S128,
        SlabSize::S256,
        SlabSize::S512,
    ];

    /// Pick the smallest slab that fits `bytes`.
    pub const fn for_bytes(bytes: usize) -> SlabSize {
        if bytes <= 16 {
            SlabSize::S16
        } else if bytes <= 32 {
            SlabSize::S32
        } else if bytes <= 64 {
            SlabSize::S64
        } else if bytes <= 128 {
            SlabSize::S128
        } else if bytes <= 256 {
            SlabSize::S256
        } else {
            SlabSize::S512
        }
    }

    /// Number of slots that fit in a 4 KiB page.
    pub const fn slots_per_page(self) -> u32 {
        4096 / self.0
    }
}
