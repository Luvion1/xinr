//! Example: cooperative scheduler using fibers, parking, and a waker.
//!
//! Run with: `cargo run --features alloc --example scheduler_demo`

#![cfg(feature = "alloc")]

use xinr::sync::fiber::Fiber;
use xinr::sync::fiber::state::FiberState;
use xinr::sync::parking::lot::ParkingLot;
use xinr::sync::scheduler::Scheduler;
use xinr::sync::timer::TimerWheel;
use xinr::sync::waker::Waker;

fn main() {
    // ---- 1. Round-robin scheduler with 3 fibers ----
    let mut sched = Scheduler::new();
    let f0 = sched.register().expect("register 0");
    let f1 = sched.register().expect("register 1");
    let f2 = sched.register().expect("register 2");
    println!("Scheduler: registered 3 fibers (ids {} {} {})", f0, f1, f2);

    // Run each fiber once via round-robin.
    for round in 0..3 {
        let id = sched.run_next().expect("run_next");
        println!("  round {}: ran fiber {}", round, id);
    }
    println!("Round-robin cycled through all 3 fibers.\n");

    // ---- 2. Park / unpark lifecycle ----
    let id = sched.register().expect("register 3");
    sched.park_current(id, 0xABCD).expect("park");
    println!("Fiber {} parked with token {:#x}", id, 0xABCDu64);
    println!("  state: {:?}", sched.state(id));
    sched.unpark(id).expect("unpark");
    println!("  state: {:?}", sched.state(id));
    sched.finish(id).expect("finish");
    println!(
        "  state: {:?} (terminal: {})",
        sched.state(id),
        sched.state(id).unwrap().is_terminal()
    );

    // ---- 3. Parking lot + timer wheel + waker integration ----
    let mut lot = ParkingLot::new();
    let mut wheel = TimerWheel::new();
    let mut waker = Waker::new();

    // Schedule a wakeup at t=100, with thread 7 parked at token 0x1.
    let permit = lot.acquire_permit();
    let parking_tok = lot.park(7, permit, 0).expect("park thread 7");
    wheel.schedule(100, 0xDEAD);
    waker.register(0xDEAD, parking_tok);
    println!(
        "\nScheduled wakeup: wheel token 0xDEAD -> parking token {:#x}",
        parking_tok
    );
    println!(
        "  lot parked: {}, wheel count: {}, waker count: {}",
        lot.parked_count(),
        wheel.count(),
        waker.count()
    );

    let woke = waker.drive(&mut wheel, &mut lot, 100);
    println!("Drove wheel to t=100: {} threads woken", woke);
    println!("  lot parked: {}", lot.parked_count());

    // ---- 4. Fiber manual lifecycle ----
    let mut f = Fiber::new(0xFEED);
    println!("\nFiber {} new state: {:?}", f.id.0, f.state);
    f.unpark();
    println!("  after unpark: {:?}", f.state);
    f.park(0x99);
    println!(
        "  after park(0x99): {:?}, token={:#x}",
        f.state,
        f.park_token()
    );
    f.unpark();
    f.finish();
    println!(
        "  after finish: {:?} (is_terminal: {})",
        f.state,
        f.state.is_terminal()
    );

    // ---- 5. Scheduler round-robin after some fibers finish ----
    for _ in 0..3 {
        let _ = sched.run_next();
    }
    println!(
        "\nScheduler after 3 more cycles; state of fiber 0: {:?}",
        sched.state(0).map(|s| s.label()).unwrap_or("none")
    );

    let _terminal: FiberState = FiberState::Finished;
    println!("\nScheduler demo complete.");
}
