//! Thread barrier tests.

use crate::sync::barrier::Barrier;

#[test]
fn barrier_all_arrive() {
    let mut b = Barrier::new(3);
    assert_eq!(b.parties(), 3);
    assert!(!b.wait().unwrap(), "first arrival: not leader");
    assert!(!b.wait().unwrap(), "second arrival: not leader");
    assert!(b.wait().unwrap(), "third arrival: leader");
}

#[test]
fn barrier_generation_advances() {
    let mut b = Barrier::new(2);
    let g0 = b.generation();
    b.wait().unwrap();
    b.wait().unwrap();
    assert!(b.generation() > g0, "generation advanced after release");
}

#[test]
fn barrier_reset() {
    let mut b = Barrier::new(2);
    b.reset();
    assert_eq!(b.waiting(), 0);
}
