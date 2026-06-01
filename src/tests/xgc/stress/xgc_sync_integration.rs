//! Integration: XGC + scope + parking lot.

use crate::sync::fiber::Fiber;
use crate::sync::parking::lot::ParkingLot;
use crate::sync::scope::Scope;
use crate::xgc::Xgc;
use crate::xgc::colored::{Color, ColoredPtr};

#[test]
fn xgc_with_scope_tracks_work() {
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    let mut scope: Scope<4> = Scope::new();
    let mut lot = ParkingLot::new();

    // Allocate some roots.
    gc.begin_mark().unwrap();
    for i in 0..4u64 {
        gc.push_root(ColoredPtr::new((0x10000 * (i + 1)) as usize, Color::White))
            .unwrap();
        let _t = scope.try_spawn::<u32>().unwrap();
        let permit = lot.acquire_permit();
        let token = lot.park(i, permit, i * 10).unwrap();
        assert_eq!(lot.parked_count(), (i + 1) as usize);
        let _ = token;
    }
    while gc.pop_work().is_some() {}
    gc.finish_mark();

    // Drain the parking lot.
    let mut woke = 0;
    for i in 0..4u64 {
        if lot.unpark_thread(i).is_some() {
            woke += 1;
        }
    }
    assert_eq!(woke, 4);
    assert_eq!(lot.parked_count(), 0);

    // Scope closes cleanly.
    while scope.active() > 0 {
        // Each active task completes (id 0..4).
        for i in 0..4 {
            scope.complete(i);
        }
    }
    scope.close().unwrap();
    gc.shutdown().unwrap();
}

#[test]
fn fiber_park_via_parking_lot() {
    use crate::sync::fiber::state::FiberState;
    let mut f = Fiber::new(99);
    f.start();
    assert_eq!(f.state, FiberState::Running);

    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    let token = lot.park(f.id.0, permit, 0).unwrap();
    f.park(token);
    assert_eq!(f.state, FiberState::Parked);
    assert_eq!(f.park_token(), token);

    lot.unpark(token).unwrap();
    f.unpark();
    assert_eq!(f.state, FiberState::Running);
    f.finish();
    assert!(f.state.is_terminal());
}
