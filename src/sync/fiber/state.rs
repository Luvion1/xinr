//! Fiber execution state.

/// State of a fiber in the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    /// Created but not yet started.
    Ready,
    /// Currently executing.
    Running,
    /// Yielded or parked, waiting to be resumed.
    Parked,
    /// Completed and joined.
    Finished,
}

impl FiberState {
    /// Short string label.
    pub fn label(self) -> &'static str {
        match self {
            FiberState::Ready => "ready",
            FiberState::Running => "running",
            FiberState::Parked => "parked",
            FiberState::Finished => "finished",
        }
    }

    /// Whether the fiber is in a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, FiberState::Finished)
    }
}
