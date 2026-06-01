# ADR-010: RwLock and Timer Wheel

## Status

Accepted, 2026-06-01.

## Context

The structured concurrency layer needed:
- A reader-writer lock for protecting read-heavy data structures (e.g.
  configuration, registry tables) where multiple readers can run in
  parallel.
- A timing wheel for scheduling delayed wakeups, parking expiry, and
  GC cycle triggers.

## Decision

### 1. RwLock (`sync::rwlock`)

- `RwState` — uses `UnsafeCell<i32>` for the reader count so multiple
  read guards can coexist (each holds `&self`, not `&mut self`).
- The writer field is `u64` (0 = free) and can only be modified via
  `&mut self` (exclusive).
- `try_read(&self) -> ReadGuard<'_>` — increments the cell, returns guard.
- `try_write(&mut self, id) -> WriteGuard<'_>` — exclusive write.
- Guards require explicit `release()` instead of `Drop` so the borrow
  checker does not extend the lock borrow until end of scope (which
  would prevent any other method from being called on the lock).

The `UnsafeCell` is single-threaded; concurrent access would need
`AtomicI32` instead, deferred until multi-threaded support lands.

### 2. Timer wheel (`sync::timer`)

- `TimerWheel` — 64-slot linear wheel, O(1) schedule, O(n) advance.
- Each slot has at most one timer; full slot retries the next slot.
- `schedule(fire_at, token) -> bool` — returns false if at capacity.
- `advance(now) -> [Option<u64>; 64]` — returns fired tokens.
- `cancel(token) -> bool` — explicit cancel by token.

Hierarchical wheels (Linux kernel style) are deferred; the linear wheel
is sufficient for the current scope (cycle triggers, parking expiry).

## Consequences

Positive:
- Multiple readers can hold guards simultaneously.
- The writer path is exclusive and obvious.
- The timer wheel has bounded memory and bounded advance cost.
- `release()` semantics are explicit, matching C/C++ lock APIs.

Negative:
- `UnsafeCell` is not `Sync`; the lock is single-threaded.
- Guards must be released manually — forgetting `release()` is a leak.
- The linear wheel has O(n) advance; for a 64-slot wheel this is fine
  but a hierarchical wheel would be needed at higher scale.
- No `Drop` on guards means partial moves are possible if the user
  takes ownership of fields.

## Alternatives Considered

- **Mutex** for the lock: rejected — read-heavy workloads want
  parallelism.
- **`parking_lot` crate**: rejected — adds a dependency and is not
  `no_std`.
- **Hierarchical timer wheel**: deferred — over-engineered for current
  scale.
- **Drop-implemented guards**: rejected — Rust's borrow checker cannot
  reason about when Drop runs, leading to "borrow extends to end of
  scope" errors in the API.
- **Heap-allocated guards**: rejected — adds an allocator dependency.
