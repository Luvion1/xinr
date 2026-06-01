//! Work-stealing scheduler: per-worker deques and a worker pool.

pub mod deque;
pub mod pool;

pub use deque::WorkDeque;
pub use pool::WorkerPool;
