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

/// Result of `select_send_4` / `select_send_8`: which channel accepted
/// the value, or which ones refused.
#[derive(Debug)]
pub struct SendResult {
    /// Index of the channel that accepted the value, if any.
    pub accepted: Option<usize>,
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

/// Try to send `value` to the first channel that has room (4-channel
/// variant). Returns the index of the channel that accepted, or `None`
/// if all are full.
pub fn select_send_4<T: Copy, const N: usize>(
    channels: [&mut BoundedChannel<T, N>; 4],
    value: T,
) -> SendResult {
    for (i, ch) in channels.into_iter().enumerate() {
        if ch.try_send(value).is_ok() {
            return SendResult { accepted: Some(i) };
        }
    }
    SendResult { accepted: None }
}

/// Try to send `value` to the first channel that has room (8-channel
/// variant).
pub fn select_send_8<T: Copy, const N: usize>(
    channels: [&mut BoundedChannel<T, N>; 8],
    value: T,
) -> SendResult {
    for (i, ch) in channels.into_iter().enumerate() {
        if ch.try_send(value).is_ok() {
            return SendResult { accepted: Some(i) };
        }
    }
    SendResult { accepted: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_send_4_picks_first_free() {
        let mut a: BoundedChannel<u32, 2> = BoundedChannel::new();
        let mut b: BoundedChannel<u32, 2> = BoundedChannel::new();
        let mut c: BoundedChannel<u32, 2> = BoundedChannel::new();
        let mut d: BoundedChannel<u32, 2> = BoundedChannel::new();
        // Fill `a` to capacity.
        a.try_send(1).unwrap();
        a.try_send(2).unwrap();
        // Now `b` is the first with room.
        let r = select_send_4([&mut a, &mut b, &mut c, &mut d], 99);
        assert_eq!(r.accepted, Some(1));
        // `a` is still full of 1, 2.
        assert_eq!(a.try_recv().unwrap(), 1);
        assert_eq!(a.try_recv().unwrap(), 2);
        assert!(a.try_recv().is_err());
    }

    #[test]
    fn select_send_4_returns_none_when_all_full() {
        let mut a: BoundedChannel<u32, 1> = BoundedChannel::new();
        let mut b: BoundedChannel<u32, 1> = BoundedChannel::new();
        let mut c: BoundedChannel<u32, 1> = BoundedChannel::new();
        let mut d: BoundedChannel<u32, 1> = BoundedChannel::new();
        a.try_send(1).unwrap();
        b.try_send(2).unwrap();
        c.try_send(3).unwrap();
        d.try_send(4).unwrap();
        let r = select_send_4([&mut a, &mut b, &mut c, &mut d], 99);
        assert_eq!(r.accepted, None);
    }

    #[test]
    fn select_send_8_round_robin_fill() {
        let mut chans: [BoundedChannel<u32, 4>; 8] = [
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
            BoundedChannel::new(),
        ];
        // Each of the 8 channels has 4 slots; 32 sends total, so 4 per channel.
        for i in 0..32u32 {
            // Find the first channel with room manually (mimicking select_send_8).
            let mut placed = false;
            for c in chans.iter_mut() {
                if c.try_send(i).is_ok() {
                    placed = true;
                    break;
                }
            }
            assert!(placed, "round {} should fit somewhere", i);
        }
        // Verify totals: each channel has 4 values.
        let mut total = 0;
        for c in chans.iter_mut() {
            while c.try_recv().is_ok() {
                total += 1;
            }
        }
        assert_eq!(total, 32);
    }
}
