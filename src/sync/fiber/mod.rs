//! Fiber: cooperative execution context with explicit yield points.

pub mod stack;
pub mod state;

use crate::sync::fiber::stack::{DEFAULT_STACK_SIZE, FiberStack};
use crate::sync::fiber::state::FiberState;

/// Fiber identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FiberId(pub u64);

/// Fiber handle.
pub struct Fiber {
    pub id: FiberId,
    pub state: FiberState,
    pub stack: FiberStack,
    /// Parent scope id (None for root fibers).
    pub parent: Option<u32>,
    /// Optional park token (set when Parked).
    park_token: u64,
}

impl Fiber {
    /// Construct a fresh fiber in the Ready state.
    pub fn new(id: u64) -> Self {
        let mut stack = FiberStack::new();
        stack.init();
        Self {
            id: FiberId(id),
            state: FiberState::Ready,
            stack,
            parent: None,
            park_token: 0,
        }
    }

    /// Construct a child fiber attached to a parent scope.
    pub fn child(id: u64, parent_scope: u32) -> Self {
        let mut f = Self::new(id);
        f.parent = Some(parent_scope);
        f
    }

    /// Transition Ready -> Running.
    pub fn start(&mut self) {
        debug_assert_eq!(
            self.state,
            FiberState::Ready,
            "fiber must be Ready to start"
        );
        self.state = FiberState::Running;
    }

    /// Yield to the scheduler. Ready or Running -> Parked.
    pub fn park(&mut self, token: u64) {
        debug_assert!(
            self.state != FiberState::Finished,
            "fiber must not be Finished to park"
        );
        self.park_token = token;
        self.state = FiberState::Parked;
    }

    /// Resume from Parked (or Ready). Parked/Ready -> Running.
    pub fn unpark(&mut self) {
        debug_assert!(
            self.state != FiberState::Finished,
            "fiber must not be Finished to unpark"
        );
        self.park_token = 0;
        self.state = FiberState::Running;
    }

    /// Mark the fiber as completed. Running -> Finished.
    pub fn finish(&mut self) {
        debug_assert_eq!(
            self.state,
            FiberState::Running,
            "fiber must be Running to finish"
        );
        self.state = FiberState::Finished;
    }

    /// Current park token. Returns 0 if not parked.
    pub fn park_token(&self) -> u64 {
        self.park_token
    }

    /// Whether the fiber can be parked.
    pub fn is_parkable(&self) -> bool {
        self.state == FiberState::Running
    }

    /// Whether the fiber can be resumed.
    pub fn is_resumable(&self) -> bool {
        self.state == FiberState::Parked
    }

    /// Stack top pointer.
    pub fn stack_top(&self) -> *mut u8 {
        self.stack.top()
    }

    /// Stack bottom pointer.
    pub fn stack_bottom(&self) -> *mut u8 {
        self.stack.bottom()
    }

    /// Stack size in bytes.
    pub fn stack_size(&self) -> usize {
        DEFAULT_STACK_SIZE
    }
}
