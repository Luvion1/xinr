//! Bulk join helpers: `count_ready` (no_std) and
//! `try_join_all` / `try_join_all_with_timeout` (alloc-gated).
//!
//! These free functions operate on a fixed-size array of `Task<T>` values
//! (as produced by `Scope::try_spawn`).

extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::RuntimeError;
use crate::sync::scope::Task;
#[cfg(feature = "alloc")]
use crate::sync::timed_join::TimedJoin;
#[cfg(feature = "alloc")]
use crate::sync::timer::TimerWheel;

/// Result of a `try_join_all` call.
#[cfg(feature = "alloc")]
pub enum JoinAll<T> {
    /// All tasks completed; carries their values in order.
    Ready(Vec<T>),
    /// At least one task has not yet completed. Callers should retry
    /// after the task(s) make progress.
    Pending,
}

/// Try to join every task. If all are ready, returns `Ready(Vec<T>)`.
/// Otherwise returns `Pending`.
#[cfg(feature = "alloc")]
pub fn try_join_all<T, const N: usize>(tasks: &mut [Task<T>; N]) -> JoinAll<T> {
    let mut out = Vec::with_capacity(N);
    for t in tasks.iter_mut() {
        match t.try_join() {
            Ok(v) => out.push(v),
            Err(RuntimeError::WouldBlock) => return JoinAll::Pending,
            Err(_) => return JoinAll::Pending,
        }
    }
    JoinAll::Ready(out)
}

/// Count tasks that have completed (without consuming them).
pub fn count_ready<T, const N: usize>(tasks: &mut [Task<T>; N]) -> usize {
    let mut n = 0;
    for t in tasks.iter_mut() {
        if t.is_done() {
            n += 1;
        }
    }
    n
}

/// Try to join with a timer wheel. Polls each task once; for ones that
/// are not yet ready, reports `TimedJoin::Timeout` in the output vector.
/// A more sophisticated version would advance the wheel and retry, but
/// for now this is a best-effort snapshot.
#[cfg(feature = "alloc")]
pub fn try_join_all_with_timeout<T, const N: usize>(
    tasks: &mut [Task<T>; N],
    wheel: &mut TimerWheel,
    deadline: u64,
) -> Vec<TimedJoin<T>>
where
    T: Default,
{
    let _ = wheel;
    let _ = deadline;
    let mut out = Vec::with_capacity(N);
    for t in tasks.iter_mut() {
        match t.try_join() {
            Ok(v) => out.push(TimedJoin::Ready(v)),
            Err(_) => out.push(TimedJoin::Timeout),
        }
    }
    out
}
