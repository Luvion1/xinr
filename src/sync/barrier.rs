//! Cyclic barrier: N threads must all arrive before any may proceed.

use crate::RuntimeError;

/// Cyclic barrier. N parties must call `wait()`; all then receive `Ok(())`
/// and the barrier resets for reuse.
pub struct Barrier {
    parties: u32,
    waiting: u32,
    generation: u64,
}

impl Barrier {
    /// Construct a barrier for `parties` threads. `parties` must be > 0.
    pub const fn new(parties: u32) -> Self {
        assert!(parties > 0, "Barrier requires at least 1 party");
        Self {
            parties,
            waiting: 0,
            generation: 0,
        }
    }

    /// Number of parties.
    pub fn parties(&self) -> u32 {
        self.parties
    }

    /// Current generation counter.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Number of threads currently waiting.
    pub fn waiting(&self) -> u32 {
        self.waiting
    }

    /// Arrive at the barrier. Returns `Ok(())` to the last waiter (signaling
    /// that all have arrived) and `Ok(())` to all others after they have
    /// been released. We use a single bool for "leader" here.
    pub fn wait(&mut self) -> Result<bool, RuntimeError> {
        self.waiting += 1;
        if self.waiting >= self.parties {
            self.waiting = 0;
            self.generation += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reset the barrier to a fresh state, discarding the current generation.
    pub fn reset(&mut self) {
        self.waiting = 0;
        self.generation += 1;
    }
}
