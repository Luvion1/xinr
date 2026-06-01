//! Round-robin scheduler: cycles through runnable fibers in order.

use crate::RuntimeError;
use crate::sync::fiber::Fiber;
use crate::sync::fiber::state::FiberState;

const MAX_FIBERS: usize = 16;

/// Round-robin scheduler. Maintains a fixed array of fiber pointers.
pub struct Scheduler {
    fibers: [Option<Fiber>; MAX_FIBERS],
    cursor: usize,
    count: u32,
}

impl Scheduler {
    /// Construct an empty scheduler.
    pub const fn new() -> Self {
        Self {
            fibers: [const { None }; MAX_FIBERS],
            cursor: 0,
            count: 0,
        }
    }

    /// Number of registered fibers.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Maximum capacity.
    pub const fn capacity(&self) -> usize {
        MAX_FIBERS
    }

    /// Register a fiber at the next available slot.
    pub fn register(&mut self) -> Result<u8, RuntimeError> {
        if (self.count as usize) >= MAX_FIBERS {
            return Err(RuntimeError::WouldBlock);
        }
        for (i, slot) in self.fibers.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(Fiber::new(i as u64));
                self.count += 1;
                return Ok(i as u8);
            }
        }
        Err(RuntimeError::WouldBlock)
    }

    /// Run the next ready fiber. Returns the id of the fiber that ran.
    /// Returns `WouldBlock` if no fibers are ready.
    pub fn run_next(&mut self) -> Result<u8, RuntimeError> {
        let n = self.count as usize;
        if n == 0 {
            return Err(RuntimeError::WouldBlock);
        }
        for offset in 0..n {
            let idx = (self.cursor + offset) % n;
            if let Some(f) = self.fibers[idx].as_ref()
                && (f.state == FiberState::Ready || f.state == FiberState::Running)
            {
                self.cursor = (idx + 1) % n;
                return Ok(idx as u8);
            }
        }
        Err(RuntimeError::WouldBlock)
    }

    /// Park the currently running fiber.
    pub fn park_current(&mut self, id: u8, token: u64) -> Result<(), RuntimeError> {
        if let Some(f) = self.fibers[id as usize].as_mut() {
            f.park(token);
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Unpark a fiber by id.
    pub fn unpark(&mut self, id: u8) -> Result<(), RuntimeError> {
        if let Some(f) = self.fibers[id as usize].as_mut() {
            f.unpark();
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Finish a fiber by id.
    pub fn finish(&mut self, id: u8) -> Result<(), RuntimeError> {
        if let Some(f) = self.fibers[id as usize].as_mut() {
            f.finish();
            Ok(())
        } else {
            Err(RuntimeError::WouldBlock)
        }
    }

    /// Get a fiber's state by id.
    pub fn state(&self, id: u8) -> Option<FiberState> {
        self.fibers[id as usize].as_ref().map(|f| f.state)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
