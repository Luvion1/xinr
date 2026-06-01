//! Tests for `join_all` bulk-join helpers.

extern crate alloc;
use crate::sync::join_all::count_ready;
#[cfg(feature = "alloc")]
use crate::sync::join_all::{JoinAll, try_join_all, try_join_all_with_timeout};
use crate::sync::scope::Scope;
#[cfg(feature = "alloc")]
use crate::sync::timer::TimerWheel;

#[cfg(feature = "alloc")]
#[test]
fn join_all_pending_when_no_tasks_completed() {
    let mut scope: Scope<4> = Scope::new();
    let mut tasks = [
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
    ];
    match try_join_all(&mut tasks) {
        JoinAll::Pending => {}
        _ => panic!("expected Pending for tasks with no values"),
    }
}

#[test]
fn count_ready_zero_for_fresh_scope() {
    let mut scope: Scope<3> = Scope::new();
    let mut tasks = [
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
    ];
    assert_eq!(count_ready(&mut tasks), 0);
}

#[cfg(feature = "alloc")]
#[test]
fn try_join_all_with_timeout_reports_timeouts() {
    let mut scope: Scope<2> = Scope::new();
    let mut tasks = [
        scope.try_spawn::<u32>().unwrap(),
        scope.try_spawn::<u32>().unwrap(),
    ];
    let mut wheel = TimerWheel::new();
    let results = try_join_all_with_timeout(&mut tasks, &mut wheel, 0);
    assert_eq!(results.len(), 2);
    // All should be Timeout because no values were sent.
    assert!(matches!(
        results[0],
        crate::sync::timed_join::TimedJoin::Timeout
    ));
    assert!(matches!(
        results[1],
        crate::sync::timed_join::TimedJoin::Timeout
    ));
}
