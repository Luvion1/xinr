//! Example: simulated GC worker using fibers, channels, and XGC.
//!
//! Demonstrates the integration of XGC with the sync primitives: a fiber
//! runs mark cycles, push work via channels, log events.
//!
//! Run with: `cargo run --features alloc --example gc_worker`

#![cfg(feature = "alloc")]

use xinr::sync::channel::BoundedChannel;
use xinr::sync::fiber::Fiber;
use xinr::sync::scope::Scope;
use xinr::sync::timer::TimerWheel;
use xinr::xgc::Xgc;
use xinr::xgc::colored::{Color, ColoredPtr};
use xinr::xgc::log::event::EventKind;
use xinr::xgc::log::ring::EventLog;

fn main() {
    println!("=== GC worker simulation ===\n");

    // ---- 1. XGC setup ----
    let mut gc = Xgc::new(8).unwrap();
    gc.init().unwrap();
    println!("[gc] initialized with 8 regions");

    // ---- 2. Worker scope (2 GC worker tasks) ----
    let mut scope: Scope<2> = Scope::new();
    let t1 = scope.try_spawn::<u32>().unwrap();
    let t2 = scope.try_spawn::<u32>().unwrap();
    println!(
        "[scope] spawned 2 worker tasks (ids {} {})",
        t1.id(),
        t2.id()
    );

    // ---- 3. Event log for diagnostics ----
    let mut log = EventLog::new();
    log.record(EventKind::Init, 0, 0);
    log.record(EventKind::MarkStart, 1, 8);
    log.record(EventKind::MarkEnd, 2, 0);
    log.record(EventKind::RelocateStart, 3, 0);
    log.record(EventKind::RelocateEnd, 4, 0);
    log.record(EventKind::SweepStart, 5, 0);
    log.record(EventKind::SweepEnd, 6, 0);
    println!("[log] recorded {} events", log.total());

    // ---- 4. Work channel ----
    let mut work: BoundedChannel<ColoredPtr, 4> = BoundedChannel::new();
    work.try_send(ColoredPtr::new(0x1000, Color::White))
        .unwrap();
    work.try_send(ColoredPtr::new(0x2000, Color::White))
        .unwrap();
    println!("[work] queued {} items", work.len());

    // ---- 5. Timer wheel for cycle scheduling ----
    let mut wheel = TimerWheel::new();
    wheel.schedule(100, 0xC1);
    wheel.schedule(200, 0xC2);
    wheel.schedule(300, 0xC3);
    println!("[timer] scheduled {} timers", wheel.count());

    // ---- 6. Drive XGC mark phase ----
    gc.begin_mark().unwrap();
    println!("[gc] mark phase started");
    // Drain work into the GC.
    while let Ok(p) = work.try_recv() {
        println!("  trace: {:#x} ({:?})", p.addr(), p.color());
    }
    gc.finish_mark();
    println!("[gc] mark phase finished");

    // ---- 7. Drive timer wheel ----
    let fired = wheel.advance(250);
    let n = fired.iter().filter(|x| x.is_some()).count();
    println!("[timer] fired {} timers at t=250", n);

    // ---- 8. Worker fibers execute ----
    let mut f1 = Fiber::new(0xF1);
    let mut f2 = Fiber::new(0xF2);
    f1.unpark();
    f2.unpark();
    f1.park(0x42);
    f2.park(0x43);
    f1.unpark();
    f2.unpark();
    f1.finish();
    f2.finish();
    println!("[fiber] both workers finished");

    // ---- 9. Drain log ----
    println!("\n[log] event timeline:");
    for ev in log.iter_chrono() {
        println!("  t={:4}  {}  value={}", ev.t_ms, ev.kind.label(), ev.value);
    }

    // ---- 10. Close scope and shutdown ----
    scope.complete(t1.id());
    scope.complete(t2.id());
    scope.close().unwrap();
    println!("\n[scope] closed at generation {}", scope.generation());

    gc.shutdown().unwrap();
    println!("[gc] shutdown complete");

    println!("\n=== GC worker demo complete ===");
}
