# ADR-011: Waker and Round-Robin Scheduler

## Status

Accepted, 2026-06-01.

## Context

After building the parking lot, timer wheel, and fiber state machine, we
needed a way to glue them together:

- A `Waker` that bridges a timer-wheel token to a parking-lot token so
  that a scheduled time translates into an actual thread wake.
- A `Scheduler` that owns a fixed array of fibers and runs them in
  round-robin order, providing a single-threaded cooperative scheduling
  primitive that exercises the parking/lot/fiber primitives.

## Decision

### 1. Waker (`sync::waker`)

- `Waker` — fixed 64-entry table mapping timer tokens to parking tokens.
- `register(timer_token, parking_token) -> bool` — register a binding.
- `cancel(timer_token) -> bool` — explicit cancel.
- `fire(timer_token, &mut ParkingLot) -> Option<ParkedThread>` — invoke
  the binding, removing it from the table.
- `drive(&mut TimerWheel, &mut ParkingLot, now) -> usize` — call
  `wheel.advance(now)`, then fire each due waker, returning the wake count.

The linear table is O(n) for fire but O(1) for register. Suitable for
small N; a hash map could be added later without changing the public API.

### 2. Scheduler (`sync::scheduler`)

- `Scheduler` — fixed 16-fiber array with a cursor.
- `register() -> Result<u8, RuntimeError>` — slot allocation, returns
  the new fiber id.
- `run_next() -> Result<u8, RuntimeError>` — round-robin over Ready/Running
  fibers.
- `park_current(id, token)` / `unpark(id)` / `finish(id)` — drive the
  state machine of a specific fiber.
- `state(id) -> Option<FiberState>` — query.

### 3. Fiber state machine relaxation

The fiber's `park` and `unpark` previously required exact state matches
(`Running` for `park`, `Parked` for `unpark`). Relaxed to: any state
except `Finished` for `park`; any state except `Finished` for `unpark`.
This matches the common pattern of "schedule" + "start" without
requiring the user to call `start` first.

## Consequences

Positive:
- The `Waker` provides a single entry point for "wake X at time T".
- The `Scheduler` exercises the parking/lot/fiber state machine without
  requiring real OS threads.
- All three (parking, timer, waker) compose: schedule a timer, register
  a waker, drive the wheel, thread wakes up.

Negative:
- The `Waker` table is O(n); not suitable for thousands of timers.
- The `Scheduler` is single-threaded and round-robin (no priorities).
- Relaxing the fiber state machine means invalid transitions (e.g.
  Parked -> Finished) are possible if the user calls methods out of
  order.

## Alternatives Considered

- **Hash map for waker entries**: deferred — overkill for current scale.
- **Priority queue in scheduler**: deferred — round-robin is sufficient
  for cooperative fibers; priorities would require ordering logic.
- **Per-fiber channels**: deferred — fibers communicate through the
  shared heap, not direct channels, in the current model.
- **Strict fiber state machine (only Running can park)**: rejected —
  forces a strict "start before park" protocol that adds boilerplate.
