//! Time source: monotonic clock for GC cycle timing.

/// Monotonic millisecond counter. Abstraction over the platform clock
/// (deferred to platform-specific implementation).
#[derive(Debug, Clone, Copy, Default)]
pub struct Instant {
    ms: u64,
}

impl Instant {
    /// Sentinel for "no time recorded".
    pub const ZERO: Instant = Instant { ms: 0 };

    /// Construct from a raw millisecond value.
    pub const fn from_ms(ms: u64) -> Self {
        Self { ms }
    }

    /// Milliseconds since the epoch (or relative to a reference point).
    pub fn ms(&self) -> u64 {
        self.ms
    }

    /// Compute elapsed milliseconds between two instants.
    pub fn elapsed_ms(self, later: Instant) -> u64 {
        later.ms.saturating_sub(self.ms)
    }
}

/// Difference between two instants, in milliseconds.
pub fn duration_ms(start: Instant, end: Instant) -> u64 {
    start.elapsed_ms(end)
}
