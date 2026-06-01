//! RwLock and timer tests.

use crate::sync::rwlock::RwLock;
use crate::sync::timer::TimerWheel;

// --- RwLock ---

#[test]
fn rwlock_new_is_free() {
    let lk = RwLock::new();
    assert!(lk.is_free());
    assert_eq!(lk.reader_count(), 0);
    assert_eq!(lk.writer_id(), None);
}

#[test]
fn rwlock_read_acquire() {
    let lk = RwLock::new();
    let g = lk.try_read().unwrap();
    assert_eq!(g.reader_count(), 1);
    assert!(!g.is_write_held());
    g.release();
    assert!(lk.is_free());
}

#[test]
fn rwlock_multiple_readers() {
    let lk = RwLock::new();
    let g1 = lk.try_read().unwrap();
    let g2 = lk.try_read().unwrap();
    assert_eq!(g2.reader_count(), 2);
    g1.release();
    g2.release();
}

#[test]
fn rwlock_write_blocks_readers() {
    let mut lk = RwLock::new();
    let wg = lk.try_write(1).unwrap();
    assert_eq!(wg.id(), 1);
    assert!(wg.is_write_held());
    wg.release();
    let rg = lk.try_read().unwrap();
    assert_eq!(rg.reader_count(), 1);
    rg.release();
}

#[test]
fn rwlock_write_excludes_writers() {
    let mut lk = RwLock::new();
    let wg = lk.try_write(1).unwrap();
    assert!(wg.is_write_held());
    wg.release();
    let wg2 = lk.try_write(2).unwrap();
    assert_eq!(wg2.id(), 2);
}

#[test]
fn rwlock_write_then_read() {
    let mut lk = RwLock::new();
    let wg = lk.try_write(1).unwrap();
    wg.release();
    let rg = lk.try_read().unwrap();
    rg.release();
}

// --- TimerWheel ---

#[test]
fn timer_wheel_new() {
    let w = TimerWheel::new();
    assert_eq!(w.wheel_size(), 64);
    assert_eq!(w.count(), 0);
}

#[test]
fn timer_schedule_and_count() {
    let mut w = TimerWheel::new();
    assert!(w.schedule(100, 0xAA));
    assert!(w.schedule(200, 0xBB));
    assert_eq!(w.count(), 2);
}

#[test]
fn timer_advance_fires_due() {
    let mut w = TimerWheel::new();
    w.schedule(50, 0xA);
    w.schedule(150, 0xB);
    let fired = w.advance(100);
    assert!(fired.contains(&Some(0xA)), "A should fire at t=100");
    assert!(!fired.contains(&Some(0xB)), "B not yet at t=100");
    assert_eq!(w.count(), 1);
}

#[test]
fn timer_advance_no_fire() {
    let mut w = TimerWheel::new();
    w.schedule(1000, 0xA);
    let fired = w.advance(500);
    assert!(!fired.contains(&Some(0xA)));
    assert_eq!(w.count(), 1);
}

#[test]
fn timer_cancel() {
    let mut w = TimerWheel::new();
    w.schedule(100, 0xA);
    assert!(w.cancel(0xA));
    assert!(!w.cancel(0xA), "already cancelled");
    assert_eq!(w.count(), 0);
}
