# ADR-009: Sync Primitives — Channels, Scopes, Parking, Fibers

## Status

Accepted, 2026-06-01.

## Context

The Xin runtime required a synchronization layer for cooperative fibers
and host threads. The original placeholder `Channel<T>` was unimplemented,
`Fiber` was a stack-less phantom struct, and the parking mechanism
referenced in `RuntimeError::NoParkingPermit` did not exist.

## Decision

Implement a complete synchronization layer under `xinr::sync`:

### 1. Channels (`sync::channel`, `sync::oneshot`)

- `BoundedRing<T, N>` — fixed-capacity ring buffer using `[Option<T>; N]`.
  No allocation, no_std-compatible. `try_push`/`try_pop` return
  `WouldBlock` when the buffer is full/empty and `Closed` when shut.
- `BoundedChannel<T, N>` — MPMC façade over the ring.
- `Oneshot<T>` — a `BoundedRing<T, 1>` with explicit semantics.

### 2. Scope (`sync::scope`)

- `Scope<N>` — tracks up to N child tasks via a 32-bit bitmap.
- `Task<T>` — handle with `try_join`, `is_done`, `id`.
- `close()` returns `Disconnected` if any task is still active, enforcing
  structured concurrency: the parent cannot drop children.

### 3. Barriers and semaphores (`sync::barrier`, `sync::semaphore`)

- `Barrier` — cyclic, generation-counter resets on release.
- `Semaphore` — counting, with explicit `waiting` counter for fairness
  analysis (no actual FIFO queue — that would require allocation).

### 4. Parking (`sync::parking`)

- `Permit` — opaque token issued to a thread that wants to park.
- `ParkingLot` — fixed-capacity registry of parked threads.
- `park(thread_id, permit, t)` returns the token; `unpark(token)` or
  `unpark_thread(id)` wakes a parked thread.

The parking lot backs the future `Condvar` and the `recv`/`send` blocking
variants of the channel.

### 5. Fibers (`sync::fiber`)

- `FiberStack` — embedded 64 KiB stack region with `top`/`bottom` pointers
  and a `pages()` query.
- `FiberState` — `Ready` / `Running` / `Parked` / `Finished`.
- `Fiber` — full state machine: `start`, `park(token)`, `unpark`, `finish`.
  Parent linkage via `parent` field for scope integration.

## Consequences

Positive:
- All sync primitives are `no_std`, allocation-free, and deterministic.
- The `Scope` enforces structured concurrency statically (parent cannot
  outlive children).
- The `ParkingLot` decouples the runtime from any particular OS threading
  model — host threads and fibers use the same `park`/`unpark` vocabulary.
- The channel API is `try_*` only; blocking versions are the caller's
  responsibility (using parking permits), keeping the primitives
  composable.

Negative:
- Channels with N = 0 are not supported; callers must size correctly.
- The parking lot is bounded to 64 entries; high-contention workloads may
  exhaust the lot and need a different design.
- `Oneshot` is just a `BoundedRing<T, 1>`; no `Drop` semantics for
  receiving end on drop (since `T` may be any type).
- The `Fiber` state machine is enforced via `debug_assert!`, not `unsafe`,
  so production builds can be transitioned incorrectly. A future ADR
  may harden this.

## Alternatives Considered

- **`crossbeam` direct use**: rejected — adds a dependency, not `no_std`.
- **Lock-free MPMC via atomics**: deferred — requires a CAS loop per
  operation; the ring buffer is sufficient for the current scope.
- **Unbounded channels**: rejected — back-pressure is essential to GC
  progress (an unbounded queue prevents the runtime from triggering a
  collection).
- **Automatic drop semantics**: rejected — needs `Drop` for the receiver
  end, which would require allocation or a static mut.
