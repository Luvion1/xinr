//! Collection trigger: decide when to start a new GC cycle.

/// Collection trigger decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// No collection needed.
    None,
    /// Trigger because the heap is full.
    Full,
    /// Trigger because high-watermark is exceeded.
    High,
    /// Trigger because too many objects were promoted.
    Promotion,
    /// Trigger after a certain number of allocations.
    Allocation,
}

/// Collection trigger: aggregates inputs and decides.
pub struct CollectionTrigger {
    pub max_capacity: u64,
    pub used: u64,
    pub high_watermark: f32,
    pub promotion_count: u32,
    pub alloc_count: u32,
    pub alloc_threshold: u32,
    pub promotion_threshold: u32,
}

impl CollectionTrigger {
    /// Construct a default trigger.
    pub const fn new(max_capacity: u64) -> Self {
        Self {
            max_capacity,
            used: 0,
            high_watermark: 0.8,
            promotion_count: 0,
            alloc_count: 0,
            alloc_threshold: 10_000,
            promotion_threshold: 1_000,
        }
    }

    /// Update current usage.
    pub fn update_usage(&mut self, used: u64) {
        self.used = used;
    }

    /// Record an allocation.
    pub fn record_alloc(&mut self) {
        self.alloc_count += 1;
    }

    /// Record a promotion.
    pub fn record_promotion(&mut self) {
        self.promotion_count += 1;
    }

    /// Decide whether a collection should be triggered.
    pub fn decide(&self) -> Trigger {
        if self.max_capacity > 0 && self.used >= self.max_capacity {
            return Trigger::Full;
        }
        if (self.used as f32) > (self.max_capacity as f32) * self.high_watermark {
            return Trigger::High;
        }
        if self.promotion_count >= self.promotion_threshold {
            return Trigger::Promotion;
        }
        if self.alloc_count >= self.alloc_threshold {
            return Trigger::Allocation;
        }
        Trigger::None
    }

    /// Reset all counters after a collection.
    pub fn reset(&mut self) {
        self.alloc_count = 0;
        self.promotion_count = 0;
    }
}
