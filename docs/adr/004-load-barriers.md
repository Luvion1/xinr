# ADR-004: Load Barriers for Concurrent Marking

## Status

Accepted — 2026-06-01

## Context

While the GC thread is concurrently marking, mutators continue to read and
write pointer fields. Two invariants must be maintained:

1. **Snapshot invariant**: every reference that existed at the start of the
   cycle is reachable from a root.
2. **Tri-color invariant**: black objects never point to white objects.

If either invariant is broken, the GC can collect a live object (use-after-free)
or leak garbage.

ZGC uses two complementary barriers:

- **SATB** (Snapshot-At-The-Beginning): on every mutator write, record the
  *old* pointer in a buffer so the GC treats it as a root.
- **Ref-update**: on writes during relocation, record the field address and
  old value so the GC can remap it later.

## Decision

- **Trait**: `LoadBarrier` exposes `read_barrier` and `write_barrier`.
- **SATB buffer**: per-thread bounded `SatbBuffer` (256 entries default).
- **Ref-update buffer**: per-thread bounded `RefUpdateBuffer` (128 entries).
- **Mark epoch**: monotonic 64-bit counter incremented at each cycle start.
- **Phase gating**: barriers are no-ops in `Idle` phase; cheap tests in
  `Marking` and `Relocating` phases.

## Alternatives Considered

| Approach          | Pros                   | Cons                              |
|-------------------|------------------------|-----------------------------------|
| Yuasa-style delete barrier | Simple           | Insufficient for concurrent mark  |
| Steele-style write barrier | Catches mutator writes | Doesn't preserve snapshot        |
| No barriers       | Zero overhead          | Cannot be concurrent             |

## Consequences

- All mutator pointer writes must be instrumented by the compiler (`xinc`).
- Buffer overflow returns `RuntimeError::StackOverflow` (deferred → spill).
- Each thread needs its own buffer (allocation cost amortized over cycle).
- The `read_barrier` is on the slow path; reads are otherwise unguarded
  once the mutator reads the pointer through `ColoredPtr::addr()`.
