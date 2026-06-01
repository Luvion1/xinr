//! `try_join_with_timeout`: try to join a scope task, polling with a
//! timer wheel. Returns `Ok(Some(value))` if the task completed, `Ok(None)`
//! if the timeout expired, or `Err` on other errors.
//!
//! This module demonstrates integration between `Scope`, `Oneshot`, and
//! `TimerWheel`.

use crate::RuntimeError;
use crate::sync::oneshot::Oneshot;
use crate::sync::timer::TimerWheel;

/// Result of `try_join_with_timeout`.
pub enum TimedJoin<T> {
    /// The task completed and the value is returned.
    Ready(T),
    /// The timeout expired before the task completed.
    Timeout,
}

/// Try to join a task with a timeout. The caller passes the `Oneshot`
/// receiver and a timer wheel. Polls the oneshot; if not ready, advances
/// the wheel to `deadline` and retries.
pub fn try_join_with_timeout<T>(
    oneshot: &mut Oneshot<T>,
    wheel: &mut TimerWheel,
    deadline: u64,
) -> Result<TimedJoin<T>, RuntimeError> {
    // First, check if already ready.
    if let Ok(v) = oneshot.recv() {
        return Ok(TimedJoin::Ready(v));
    }
    // Advance the wheel to the deadline.
    let _ = wheel.advance(deadline);
    // Try again.
    if let Ok(v) = oneshot.recv() {
        return Ok(TimedJoin::Ready(v));
    }
    Ok(TimedJoin::Timeout)
}
