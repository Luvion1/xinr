//! Structured concurrency: fibers, channels, and sync primitives.
//!
//! Xin runtime provides cooperative fibers, bounded MPMC channels,
//! one-shot channels, cyclic barriers, semaphores, structured scopes,
//! and a parking lot for blocking operations.
//! Designed for deterministic, low-overhead concurrency.

pub mod barrier;
pub mod blocking;
pub mod cache_padded;
pub mod channel;
pub mod condvar;
pub mod fiber;
pub mod join_all;
pub mod metrics;
pub mod notify;
pub mod oneshot;
pub mod parking;
pub mod rwlock;
pub mod scheduler;
pub mod scope;
pub mod select;
pub mod select_macro;
pub mod semaphore;
pub mod spsc;
pub mod timed_join;
pub mod timer;
pub mod waitgroup;
pub mod waker;

pub use barrier::Barrier;
pub use blocking::{BlockingChannel, RecvOutcome};
pub use cache_padded::{CACHE_LINE, CachePadded, PaddedCounter};
pub use channel::BoundedChannel;
pub use channel::ring::BoundedRing;
pub use condvar::Condvar;
pub use fiber::stack::{DEFAULT_STACK_SIZE, FiberStack};
pub use fiber::state::FiberState;
pub use fiber::{Fiber, FiberId};
pub use join_all::count_ready;
#[cfg(feature = "alloc")]
pub use join_all::{JoinAll, try_join_all, try_join_all_with_timeout};
pub use metrics::Metrics;
pub use notify::Notify;
pub use oneshot::Oneshot;
pub use parking::lot::{ParkedThread, ParkingLot};
pub use parking::permit::Permit;
pub use rwlock::RwLock;
pub use rwlock::guard::{ReadGuard, WriteGuard};
pub use rwlock::state::RwState;
pub use scheduler::Scheduler;
pub use scope::{JoinState, Scope, Task};
pub use select::{
    SelectResult, SendResult, select_recv_4, select_recv_8, select_send_4, select_send_8,
};
pub use semaphore::Semaphore;
pub use spsc::{MpscChannel, SpscChannel};
pub use timed_join::{TimedJoin, try_join_with_timeout};
pub use timer::{Timer, TimerWheel};
pub use waitgroup::WaitGroup;
pub use waker::Waker;
