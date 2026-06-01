//! Scope tests.

use crate::RuntimeError;
use crate::sync::scope::{Scope, Task};

#[test]
fn scope_new_is_empty() {
    let s: Scope<4> = Scope::new();
    assert!(s.is_empty());
    assert_eq!(s.active(), 0);
}

#[test]
fn scope_spawn_and_complete() {
    let mut s: Scope<4> = Scope::new();
    let t: Task<u32> = s.try_spawn().unwrap();
    assert_eq!(s.active(), 1);
    s.complete(t.id());
    assert!(s.is_empty());
}

#[test]
fn scope_full_rejects() {
    let mut s: Scope<2> = Scope::new();
    s.try_spawn::<u32>().unwrap();
    s.try_spawn::<u32>().unwrap();
    assert!(matches!(
        s.try_spawn::<u32>(),
        Err(RuntimeError::WouldBlock)
    ));
}

#[test]
fn scope_close_requires_empty() {
    let mut s: Scope<2> = Scope::new();
    let t = s.try_spawn::<u32>().unwrap();
    assert_eq!(s.close(), Err(RuntimeError::Disconnected));
    s.complete(t.id());
    s.close().unwrap();
}

#[test]
fn scope_task_join_unready() {
    let mut s: Scope<4> = Scope::new();
    let mut t: Task<u32> = s.try_spawn().unwrap();
    assert_eq!(t.try_join(), Err(RuntimeError::WouldBlock));
    assert!(!t.is_done());
}

#[test]
fn scope_generation_increments() {
    let mut s: Scope<2> = Scope::new();
    let g0 = s.generation();
    let t = s.try_spawn::<u32>().unwrap();
    s.complete(t.id());
    s.close().unwrap();
    assert!(s.generation() > g0);
}
