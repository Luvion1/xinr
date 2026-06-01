//! Example: producer/consumer using a bounded channel.
//!
//! Demonstrates the sync primitives (channel, fiber, scope, parking).
//!
//! Run with: `cargo run --features alloc --example channel_demo`

#![cfg(feature = "alloc")]

use xinr::RuntimeError;
use xinr::sync::channel::BoundedChannel;
use xinr::sync::fiber::state::FiberState;
use xinr::sync::fiber::{Fiber, FiberId};
use xinr::sync::parking::lot::ParkingLot;
use xinr::sync::scope::Scope;

fn main() {
    // ---- Bounded MPMC channel ----
    let mut ch: BoundedChannel<u32, 4> = BoundedChannel::new();
    for v in 1..=4 {
        ch.try_send(v * 10).expect("send");
    }
    assert!(ch.is_full());
    println!("Channel filled to capacity ({}).", ch.capacity());
    while let Ok(v) = ch.try_recv() {
        println!("  recv: {}", v);
    }

    // ---- Oneshot channel ----
    let mut once: xinr::sync::oneshot::Oneshot<&'static str> = xinr::sync::oneshot::Oneshot::new();
    once.send("hello").expect("oneshot send");
    println!("Oneshot received: {}", once.recv().expect("oneshot recv"));

    // ---- Scope with 3 tasks ----
    let mut scope: Scope<4> = Scope::new();
    let t1: xinr::sync::scope::Task<u32> = scope.try_spawn().expect("spawn 1");
    let t2: xinr::sync::scope::Task<u32> = scope.try_spawn().expect("spawn 2");
    let t3: xinr::sync::scope::Task<u32> = scope.try_spawn().expect("spawn 3");
    println!("Scope active = {}", scope.active());
    scope.complete(t1.id());
    scope.complete(t2.id());
    scope.complete(t3.id());
    scope.close().expect("close");
    println!("Scope closed at generation {}", scope.generation());

    // ---- Barrier (2 parties) ----
    let mut barrier = xinr::sync::barrier::Barrier::new(2);
    let leader = barrier.wait().expect("wait 1");
    let _ = barrier.wait().expect("wait 2");
    println!(
        "Barrier leader: {}, generation: {}",
        leader,
        barrier.generation()
    );

    // ---- Semaphore (3 permits) ----
    let mut sem = xinr::sync::semaphore::Semaphore::new(3);
    sem.try_acquire().unwrap();
    sem.try_acquire().unwrap();
    sem.try_acquire().unwrap();
    assert_eq!(sem.try_acquire(), Err(RuntimeError::WouldBlock));
    println!("Semaphore exhausted after 3 acquires.");

    // ---- Fiber state machine ----
    let mut f = Fiber::new(0xC0FFEE);
    f.start();
    f.park(0xABCD);
    println!("Fiber {} state: {}", f.id.0, f.state.label());
    f.unpark();
    f.finish();
    println!("Fiber terminated: {}", f.state.is_terminal());

    // ---- Parking lot ----
    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    let token = lot.park(42, permit, 1000).expect("park");
    println!("Parked thread 42 with token {:#x}", token);
    let rec = lot.unpark(token).expect("unpark");
    println!(
        "Unparked: thread={}, parked_at={}",
        rec.thread_id, rec.parked_at
    );

    // ---- FiberId sanity ----
    let _id: FiberId = FiberId(1);
    let _state: FiberState = FiberState::Ready;
    println!("Demo complete.");
}
