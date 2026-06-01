//! Structured concurrency scope.
//!
//! A `Scope` tracks up to `N` child tasks. The scope cannot be "closed" until
//! every child is either finished or has been forcibly detached. This is the
//! structured concurrency guarantee: parents cannot outlive their children.

use crate::RuntimeError;
use crate::sync::oneshot::Oneshot;

/// Child task handle.
pub struct Task<T> {
    id: u32,
    result: Oneshot<T>,
}

impl<T> core::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("ready", &self.result.is_ready())
            .finish()
    }
}

impl<T> Task<T> {
    /// Try to receive the task's result. Returns `WouldBlock` if not ready.
    pub fn try_join(&mut self) -> Result<T, RuntimeError> {
        self.result.recv()
    }

    /// Task id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Whether the task has completed.
    pub fn is_done(&mut self) -> bool {
        self.result.is_ready()
    }
}

/// Scope of up to `N` concurrent tasks.
pub struct Scope<const N: usize> {
    /// Bitmap of active tasks: bit `i` set means task `i` is in flight.
    active: u32,
    /// Generation counter (increments on every `close`).
    generation: u32,
}

impl<const N: usize> Scope<N> {
    /// Construct a new scope. Panics if N == 0 or N > 32.
    pub const fn new() -> Self {
        assert!(N > 0, "Scope requires capacity > 0");
        assert!(N <= 32, "Scope capacity max is 32");
        Self {
            active: 0,
            generation: 0,
        }
    }

    /// Current generation.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Number of active tasks.
    pub fn active(&self) -> u32 {
        self.active.count_ones()
    }

    /// Whether the scope is empty (all children joined).
    pub fn is_empty(&self) -> bool {
        self.active == 0
    }

    /// Try to spawn a new task. Returns `Closed` if scope is full.
    pub fn try_spawn<T>(&mut self) -> Result<Task<T>, RuntimeError> {
        for i in 0..N {
            let mask = 1u32 << i;
            if self.active & mask == 0 {
                self.active |= mask;
                return Ok(Task {
                    id: i as u32,
                    result: Oneshot::new(),
                });
            }
        }
        Err(RuntimeError::WouldBlock)
    }

    /// Mark a task as completed (called by the runtime when the task finishes).
    pub fn complete(&mut self, task_id: u32) {
        if task_id < N as u32 {
            self.active &= !(1u32 << task_id);
        }
    }

    /// Close the scope. Returns `Ok(())` if no active tasks, otherwise
    /// `Disconnected` to refuse the close.
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if self.active != 0 {
            return Err(RuntimeError::Disconnected);
        }
        self.generation += 1;
        Ok(())
    }

    /// Try to join with a timeout. Returns:
    /// - `Ok(Ready)` if the scope is empty (all tasks complete).
    /// - `Ok(Timeout)` if `max_cycles` `complete` calls have been issued
    ///   without the scope becoming empty.
    /// - `Err(Disconnected)` on error.
    ///
    /// This is a polling-style join: the caller must drive `complete()` for
    /// finished tasks. The scope's bitmap is the source of truth.
    pub fn try_join_with_timeout(&mut self, max_cycles: u32) -> Result<JoinState, RuntimeError> {
        for _ in 0..max_cycles {
            if self.active == 0 {
                return Ok(JoinState::Ready);
            }
            // Yield to the scheduler (a no-op in the no-std runtime).
        }
        Ok(if self.active == 0 {
            JoinState::Ready
        } else {
            JoinState::Timeout
        })
    }
}

/// Outcome of `try_join_with_timeout`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinState {
    /// All tasks complete.
    Ready,
    /// Timeout reached.
    Timeout,
}

impl<const N: usize> Default for Scope<N> {
    fn default() -> Self {
        Self::new()
    }
}
