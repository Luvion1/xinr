//! Timer wheel: hierarchical timing wheel for scheduled wakeups.

const WHEEL_SIZE: usize = 64;
const MAX_TIMERS: usize = 256;

/// One scheduled wakeup.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timer {
    /// Time at which to fire.
    pub fire_at: u64,
    /// Token associated with this timer.
    pub token: u64,
    /// Whether the slot is active.
    pub active: bool,
}

/// Hierarchical timing wheel.
pub struct TimerWheel {
    slots: [Timer; WHEEL_SIZE],
    cursor: u64,
    count: u32,
}

impl TimerWheel {
    /// Construct a new wheel.
    pub const fn new() -> Self {
        Self {
            slots: [Timer {
                fire_at: 0,
                token: 0,
                active: false,
            }; WHEEL_SIZE],
            cursor: 0,
            count: 0,
        }
    }

    /// Number of slots in the wheel.
    pub const fn wheel_size(&self) -> usize {
        WHEEL_SIZE
    }

    /// Number of active timers.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Schedule a timer at `fire_at` with `token`. Returns true on success.
    pub fn schedule(&mut self, fire_at: u64, token: u64) -> bool {
        if (self.count as usize) >= MAX_TIMERS {
            return false;
        }
        let offset = (fire_at - self.cursor) as usize;
        let slot = offset % WHEEL_SIZE;
        for i in 0..WHEEL_SIZE {
            let idx = (slot + i) % WHEEL_SIZE;
            if !self.slots[idx].active {
                self.slots[idx] = Timer {
                    fire_at,
                    token,
                    active: true,
                };
                self.count += 1;
                return true;
            }
        }
        false
    }

    /// Advance the wheel to `now`. Returns tokens that have fired.
    pub fn advance(&mut self, now: u64) -> [Option<u64>; WHEEL_SIZE] {
        let mut out = [None; WHEEL_SIZE];
        let mut n = 0;
        for slot in self.slots.iter_mut() {
            if slot.active && slot.fire_at <= now {
                out[n] = Some(slot.token);
                n += 1;
                *slot = Timer::default();
                self.count = self.count.saturating_sub(1);
                if n >= WHEEL_SIZE {
                    break;
                }
            }
        }
        self.cursor = now;
        out
    }

    /// Cancel a timer by token. Returns true if cancelled.
    pub fn cancel(&mut self, token: u64) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.active && slot.token == token {
                *slot = Timer::default();
                self.count = self.count.saturating_sub(1);
                return true;
            }
        }
        false
    }
}

impl Default for TimerWheel {
    fn default() -> Self {
        Self::new()
    }
}
