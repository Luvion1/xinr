//! MPMC select: poll multiple channels and return the first ready.
//!
//! Given a fixed array of `BoundedChannel<T, N>` references, `select_recv`
//! returns the index and value of the first channel that has a value
//! available, or `None` if all are empty.

use crate::RuntimeError;
use crate::sync::channel::BoundedChannel;

/// Index+value pair returned by `select_recv`.
#[derive(Debug)]
pub struct SelectResult<T> {
    pub index: usize,
    pub value: T,
}

/// Poll up to 4 channels. Returns the first one that has data.
pub fn select_recv_4<T, const N: usize>(
    channels: [&mut BoundedChannel<T, N>; 4],
) -> Result<Option<SelectResult<T>>, RuntimeError> {
    for (i, ch) in channels.into_iter().enumerate() {
        if let Ok(v) = ch.try_recv() {
            return Ok(Some(SelectResult { index: i, value: v }));
        }
    }
    Ok(None)
}

/// Poll up to 8 channels.
pub fn select_recv_8<T, const N: usize>(
    channels: [&mut BoundedChannel<T, N>; 8],
) -> Result<Option<SelectResult<T>>, RuntimeError> {
    for (i, ch) in channels.into_iter().enumerate() {
        if let Ok(v) = ch.try_recv() {
            return Ok(Some(SelectResult { index: i, value: v }));
        }
    }
    Ok(None)
}
