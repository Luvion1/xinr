//! Tunable parameters for the GC's adaptive behavior.

/// Heap sizing heuristic: when to grow, when to shrink.
#[derive(Debug, Clone, Copy)]
pub struct HeapSizing {
    /// Grow heap when usage exceeds `high_watermark * capacity`.
    pub high_watermark: f32,
    /// Shrink heap when usage drops below `low_watermark * capacity`.
    pub low_watermark: f32,
    /// Minimum growth factor (e.g. 1.25 = 25% growth).
    pub grow_factor: f32,
    /// Minimum shrink factor (e.g. 0.75 = 25% shrink).
    pub shrink_factor: f32,
}

impl HeapSizing {
    /// Conservative defaults.
    pub const fn default_conservative() -> Self {
        Self {
            high_watermark: 0.8,
            low_watermark: 0.3,
            grow_factor: 1.25,
            shrink_factor: 0.75,
        }
    }

    /// Aggressive defaults: trigger collection more often.
    pub const fn default_aggressive() -> Self {
        Self {
            high_watermark: 0.6,
            low_watermark: 0.2,
            grow_factor: 1.5,
            shrink_factor: 0.5,
        }
    }

    /// Decide if the heap should grow given current usage and capacity.
    pub fn should_grow(&self, used: u64, capacity: u64) -> bool {
        capacity > 0 && (used as f32) > (capacity as f32) * self.high_watermark
    }

    /// Decide if the heap should shrink.
    pub fn should_shrink(&self, used: u64, capacity: u64) -> bool {
        capacity > 0 && (used as f32) < (capacity as f32) * self.low_watermark
    }

    /// Compute new capacity given current capacity and a grow decision.
    pub fn grown_capacity(&self, capacity: u64) -> u64 {
        ((capacity as f32) * self.grow_factor) as u64
    }

    /// Compute new capacity given a shrink decision.
    pub fn shrunk_capacity(&self, capacity: u64) -> u64 {
        ((capacity as f32) * self.shrink_factor) as u64
    }
}
