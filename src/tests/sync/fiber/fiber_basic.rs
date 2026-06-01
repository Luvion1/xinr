//! Fiber tests.

use crate::sync::fiber::stack::{DEFAULT_STACK_SIZE, FiberStack};
use crate::sync::fiber::state::FiberState;
use crate::sync::fiber::{Fiber, FiberId};

// --- FiberState ---

#[test]
fn fiber_state_labels() {
    assert_eq!(FiberState::Ready.label(), "ready");
    assert_eq!(FiberState::Running.label(), "running");
    assert_eq!(FiberState::Parked.label(), "parked");
    assert_eq!(FiberState::Finished.label(), "finished");
}

#[test]
fn fiber_state_terminal() {
    assert!(!FiberState::Ready.is_terminal());
    assert!(!FiberState::Running.is_terminal());
    assert!(!FiberState::Parked.is_terminal());
    assert!(FiberState::Finished.is_terminal());
}

// --- FiberStack ---

#[test]
fn fiber_stack_init() {
    let mut s = FiberStack::new();
    assert_eq!(s.size(), DEFAULT_STACK_SIZE);
    assert_eq!(s.pages(), DEFAULT_STACK_SIZE / 4096);
    assert!(s.top().is_null());
    s.init();
    assert!(!s.top().is_null());
    assert!(!s.bottom().is_null());
    assert!(s.remaining() > 0);
}

#[test]
fn fiber_stack_default() {
    let s = FiberStack::default();
    assert_eq!(s.size(), DEFAULT_STACK_SIZE);
}

// --- Fiber ---

#[test]
fn fiber_new_is_ready() {
    let f = Fiber::new(1);
    assert_eq!(f.id, FiberId(1));
    assert_eq!(f.state, FiberState::Ready);
    assert!(!f.is_parkable());
    assert!(!f.is_resumable());
}

#[test]
fn fiber_full_lifecycle() {
    let mut f = Fiber::new(0);
    f.start();
    assert_eq!(f.state, FiberState::Running);
    f.park(42);
    assert_eq!(f.state, FiberState::Parked);
    assert_eq!(f.park_token(), 42);
    f.unpark();
    assert_eq!(f.state, FiberState::Running);
    assert_eq!(f.park_token(), 0);
    f.finish();
    assert!(f.state.is_terminal());
}

#[test]
fn fiber_child_has_parent() {
    let f = Fiber::child(7, 3);
    assert_eq!(f.parent, Some(3));
}

#[test]
fn fiber_park_unpark_state_machine() {
    let mut f = Fiber::new(0);
    f.start();
    assert!(f.is_parkable());
    f.park(1);
    assert!(f.is_resumable());
    assert!(!f.is_parkable());
}

#[test]
fn fiber_park_token_updates() {
    let mut f = Fiber::new(0);
    f.start();
    f.park(0xDEAD);
    assert_eq!(f.park_token(), 0xDEAD);
    f.unpark();
    f.park(0xBEEF);
    assert_eq!(f.park_token(), 0xBEEF);
}
