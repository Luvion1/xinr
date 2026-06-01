//! Example: XGC + sync integration.
//!
//! Demonstrates using scope + parking + fiber together with XGC's mark/relocate
//! cycle.
//!
//! Run with: `cargo run --features alloc --example full_pipeline`

#![cfg(feature = "alloc")]

use xinr::sync::fiber::Fiber;
use xinr::sync::fiber::state::FiberState;
use xinr::sync::parking::lot::ParkingLot;
use xinr::sync::rwlock::RwLock;
use xinr::sync::scope::Scope;
use xinr::sync::timer::TimerWheel;
use xinr::xgc::Xgc;
use xinr::xgc::colored::{Color, ColoredPtr};

fn main() {
    // ---- 1. XGC mark/relocate cycle ----
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
    println!("XGC: mark phase complete.");

    // ---- 2. Scope with 2 tasks ----
    let mut scope: Scope<2> = Scope::new();
    let t1 = scope.try_spawn::<u32>().unwrap();
    let t2 = scope.try_spawn::<u32>().unwrap();
    println!("Scope active = {}", scope.active());
    scope.complete(t1.id());
    scope.complete(t2.id());
    scope.close().unwrap();
    println!("Scope closed at generation {}", scope.generation());

    // ---- 3. Parking lot: park 2 threads then unpark ----
    let mut lot = ParkingLot::new();
    let p1 = lot.acquire_permit();
    let p2 = lot.acquire_permit();
    let t1_tok = lot.park(100, p1, 1000).unwrap();
    let t2_tok = lot.park(200, p2, 2000).unwrap();
    println!("Parking lot: parked_count = {}", lot.parked_count());
    lot.unpark(t1_tok).unwrap();
    lot.unpark(t2_tok).unwrap();
    println!("Parking lot: parked_count = {}", lot.parked_count());

    // ---- 4. Fiber state machine ----
    let mut f = Fiber::new(0xCAFE);
    f.start();
    f.park(0x42);
    println!("Fiber state: {}", f.state.label());
    f.unpark();
    f.finish();
    println!(
        "Fiber state: {} (terminal: {})",
        f.state.label(),
        f.state.is_terminal()
    );

    // ---- 5. RwLock with multiple readers ----
    let lk = RwLock::new();
    let g1 = lk.try_read().unwrap();
    let g2 = lk.try_read().unwrap();
    println!("RwLock readers: {}", g2.reader_count());
    g1.release();
    g2.release();

    // ---- 6. Timer wheel ----
    let mut wheel = TimerWheel::new();
    wheel.schedule(100, 0xAA);
    wheel.schedule(200, 0xBB);
    println!("Timer wheel: count = {}", wheel.count());
    let fired = wheel.advance(150);
    let fired_count = fired.iter().filter(|x| x.is_some()).count();
    println!("Timer wheel: fired {} timers at t=150", fired_count);

    // ---- 7. XGC shutdown ----
    gc.shutdown().unwrap();
    println!("XGC: shutdown complete.");
    println!("All subsystems exercised successfully.");

    // Force the FiberState import to be used.
    let _state: FiberState = FiberState::Ready;
}
