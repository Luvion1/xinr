//! Final stress test: full pipeline of XGC + sync.

use crate::sync::barrier::Barrier;
use crate::sync::blocking::BlockingChannel;
use crate::sync::channel::BoundedChannel;
use crate::sync::fiber::Fiber;
use crate::sync::fiber::state::FiberState;
use crate::sync::oneshot::Oneshot;
use crate::sync::parking::lot::ParkingLot;
use crate::sync::parking::permit::Permit;
use crate::sync::rwlock::RwLock;
use crate::sync::scope::Scope;
use crate::sync::semaphore::Semaphore;
use crate::sync::timer::TimerWheel;
use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn all_subsystems_in_one_test() {
    // 1. XGC
    let mut gc = Xgc::new(16).unwrap();
    gc.init().unwrap();
    gc.begin_mark().unwrap();
    for i in 0..4u64 {
        gc.push_root(ColoredPtr::new(
            ((0x10000 * (i + 1)) & 0x7FFFFFFF) as usize,
            Color::White,
        ))
        .unwrap();
    }
    while gc.pop_work().is_some() {}
    gc.finish_mark();

    // 2. Channel
    let mut ch: BoundedChannel<u32, 4> = BoundedChannel::new();
    for v in [100u32, 200, 300, 400] {
        ch.try_send(v).unwrap();
    }
    let total: u32 = (0..4).map(|_| ch.try_recv().unwrap()).sum();
    assert_eq!(total, 1000);

    // 3. Oneshot
    let mut once: Oneshot<u64> = Oneshot::new();
    once.send(0xDEAD_BEEF).unwrap();
    assert_eq!(once.recv().unwrap(), 0xDEAD_BEEF);

    // 4. Scope
    let mut scope: Scope<2> = Scope::new();
    let t1 = scope.try_spawn::<u32>().unwrap();
    let t2 = scope.try_spawn::<u32>().unwrap();
    scope.complete(t1.id());
    scope.complete(t2.id());
    scope.close().unwrap();

    // 5. Barrier
    let mut barrier = Barrier::new(2);
    assert!(!barrier.wait().unwrap());
    assert!(barrier.wait().unwrap());

    // 6. Semaphore
    let mut sem = Semaphore::new(1);
    sem.try_acquire().unwrap();
    sem.release().unwrap();

    // 7. Parking lot
    let mut lot = ParkingLot::new();
    let p = lot.acquire_permit();
    let t = lot.park(0xA, p, 0).unwrap();
    lot.unpark(t).unwrap();

    // 8. Fiber
    let mut f = Fiber::new(1);
    f.start();
    f.park(0x42);
    assert_eq!(f.state, FiberState::Parked);
    f.unpark();
    f.finish();
    assert_eq!(f.state, FiberState::Finished);

    // 9. RwLock
    let lk = RwLock::new();
    let g1 = lk.try_read().unwrap();
    let g2 = lk.try_read().unwrap();
    g1.release();
    g2.release();

    // 10. Timer
    let mut wheel = TimerWheel::new();
    wheel.schedule(100, 0xA);
    wheel.schedule(200, 0xB);
    let fired = wheel.advance(150);
    assert!(fired.contains(&Some(0xA)));
    assert!(!fired.contains(&Some(0xB)));

    // 11. Blocking channel
    let mut bch: BlockingChannel<u8, 1> = BlockingChannel::new();
    bch.try_send(0xFE).unwrap();
    let parked = bch.send(0xED, 1, 0).unwrap();
    assert!(parked.is_some());
    const { assert!(Permit::NONE.0 == 0) };

    gc.shutdown().unwrap();
}

#[test]
fn high_contention_parking_lot() {
    let mut lot = ParkingLot::new();
    let mut tokens = [0u64; 50];
    for i in 0..50u64 {
        let p = lot.acquire_permit();
        tokens[i as usize] = lot.park(i, p, i * 10).unwrap();
    }
    assert_eq!(lot.parked_count(), 50);
    for (i, &t) in tokens.iter().enumerate() {
        assert!(lot.unpark(t).is_some(), "thread {} unpark failed", i);
    }
    assert_eq!(lot.parked_count(), 0);
}

#[test]
fn rwlock_high_reader_contention() {
    let lk = RwLock::new();
    let g1 = lk.try_read().unwrap();
    let g2 = lk.try_read().unwrap();
    let g3 = lk.try_read().unwrap();
    let g4 = lk.try_read().unwrap();
    let g5 = lk.try_read().unwrap();
    let g6 = lk.try_read().unwrap();
    let g7 = lk.try_read().unwrap();
    let g8 = lk.try_read().unwrap();
    let g9 = lk.try_read().unwrap();
    let g10 = lk.try_read().unwrap();
    let all = [g1, g2, g3, g4, g5, g6, g7, g8, g9, g10];
    for g in &all {
        assert_eq!(g.reader_count(), 10);
    }
    for g in all {
        g.release();
    }
}
