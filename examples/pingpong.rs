//! Example: fiber ping-pong using channels.
//!
//! Two fibers take turns sending a value back and forth through channels.
//!
//! Run with: `cargo run --features alloc --example pingpong`

#![cfg(feature = "alloc")]

use xinr::sync::channel::BoundedChannel;
use xinr::sync::fiber::Fiber;
use xinr::sync::fiber::state::FiberState;

fn main() {
    // Two channels: A->B and B->A.
    let mut a_to_b: BoundedChannel<u32, 4> = BoundedChannel::new();
    let mut b_to_a: BoundedChannel<u32, 4> = BoundedChannel::new();

    // Seed: A starts with the ball.
    a_to_b.try_send(1).unwrap();
    println!("Starting ping-pong with value 1");

    // Two fibers alternate: A receives, increments, sends to B; B receives,
    // increments, sends to A. The game ends when value >= 6.
    let mut fiber_a = Fiber::new(1);
    let mut fiber_b = Fiber::new(2);

    // Start both fibers.
    fiber_a.unpark();
    fiber_b.unpark();

    let mut turns = 0;
    let max_turns = 10;

    while turns < max_turns {
        // Fiber A's turn.
        if let Ok(v) = a_to_b.try_recv() {
            println!("  A received {}", v);
            if v >= 6 {
                println!("A wins with value {}!", v);
                break;
            }
            b_to_a.try_send(v + 1).unwrap();
            turns += 1;
        } else {
            println!("A is blocked (no ball)");
        }

        // Fiber B's turn.
        if let Ok(v) = b_to_a.try_recv() {
            println!("  B received {}", v);
            if v >= 6 {
                println!("B wins with value {}!", v);
                break;
            }
            a_to_b.try_send(v + 1).unwrap();
            turns += 1;
        } else {
            println!("B is blocked (no ball)");
        }

        // Yield fibers between turns.
        fiber_a.park(0);
        fiber_b.park(0);
        fiber_a.unpark();
        fiber_b.unpark();
    }

    fiber_a.finish();
    fiber_b.finish();

    println!("\nFinal state:");
    println!("  fiber_a: {:?}", fiber_a.state);
    println!("  fiber_b: {:?}", fiber_b.state);
    println!("  A->B queued: {}", a_to_b.len());
    println!("  B->A queued: {}", b_to_a.len());
    println!("  Total turns: {}", turns);

    let _state: FiberState = FiberState::Ready;
    println!("\nPing-pong demo complete.");
}
