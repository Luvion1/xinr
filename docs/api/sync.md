# Sync API Reference

The `xinr::sync` module provides structured concurrency primitives for
cooperative fibers and host threads. All primitives are `no_std` and
allocation-free.

## Channel

### `BoundedRing<T, N>`

Fixed-capacity ring buffer.

```rust
use xinr::sync::channel::ring::BoundedRing;

let mut r: BoundedRing<u32, 4> = BoundedRing::new();
r.try_push(1).unwrap();
let v = r.try_pop().unwrap();
```

### `BoundedChannel<T, N>`

Multi-producer multi-consumer channel. Alias for `BoundedRing<T, N>` with
named semantics.

```rust
use xinr::sync::channel::BoundedChannel;
let mut ch: BoundedChannel<u8, 2> = BoundedChannel::new();
ch.try_send(1).unwrap();
ch.try_recv().unwrap();
```

### `Oneshot<T>`

Single-slot channel.

```rust
use xinr::sync::oneshot::Oneshot;
let mut o: Oneshot<u32> = Oneshot::new();
o.send(42).unwrap();
assert_eq!(o.recv().unwrap(), 42);
```

### `BlockingChannel<T, N>`

Blocking MPMC channel backed by a parking lot.

```rust
use xinr::sync::blocking::BlockingChannel;
let mut ch: BlockingChannel<u8, 1> = BlockingChannel::new();
ch.try_send(1).unwrap();
let permit = ch.send(2, 7, 0).unwrap().unwrap();  // parks, returns permit
ch.wake_sender(permit.0);
```

## Scope

### `Scope<N>`

Structured concurrency scope. Up to N child tasks.

```rust
use xinr::sync::scope::Scope;
let mut s: Scope<4> = Scope::new();
let t = s.try_spawn::<u32>().unwrap();
s.complete(t.id());
s.close().unwrap();  // fails if any task still active
```

### `Task<T>`

Child task handle. Use `try_join()` to get the result.

## Barriers and Semaphores

### `Barrier`

Cyclic barrier.

```rust
use xinr::sync::barrier::Barrier;
let mut b = Barrier::new(3);
b.wait().unwrap();
b.wait().unwrap();
let leader = b.wait().unwrap();  // last arrival returns true
```

### `Semaphore`

Counting semaphore.

```rust
use xinr::sync::semaphore::Semaphore;
let mut s = Semaphore::new(2);
s.try_acquire().unwrap();
s.release().unwrap();
```

## Parking

### `Permit`

Opaque token for parking.

### `ParkingLot`

Registry of parked threads.

```rust
use xinr::sync::parking::lot::ParkingLot;
let mut lot = ParkingLot::new();
let permit = lot.acquire_permit();
let token = lot.park(42, permit, 0).unwrap();
lot.unpark(token).unwrap();
```

### `Condvar`

Condition variable backed by a parking lot.

## Fibers

### `Fiber`

Cooperative execution context.

```rust
use xinr::sync::fiber::Fiber;
let mut f = Fiber::new(1);
f.start();
f.park(0x42);   // Running -> Parked
f.unpark();     // Parked -> Running
f.finish();     // Running -> Finished
```

### `FiberState`

`Ready`, `Running`, `Parked`, `Finished`.

## RwLock

Reader-writer lock. Multiple readers OR one writer.

```rust
use xinr::sync::rwlock::RwLock;
let lk = RwLock::new();
let rg = lk.try_read().unwrap();
let rg2 = lk.try_read().unwrap();  // second reader allowed
rg.release();
rg2.release();
```

## Timer

### `TimerWheel`

64-slot timing wheel.

```rust
use xinr::sync::timer::TimerWheel;
let mut w = TimerWheel::new();
w.schedule(100, 0xA);
let fired = w.advance(150);
```

## Error Variants

All primitives return `Result<_, RuntimeError>` with the following
relevant variants:

- `WouldBlock` — operation would have blocked
- `Closed` — channel/scope is closed
- `Disconnected` — sender or receiver was dropped
- `OutOfMemory` — allocation failed
- `NotInitialized` — runtime not initialized
