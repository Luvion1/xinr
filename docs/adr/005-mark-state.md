# ADR-005: Mark State Machine and Phase Coordination

## Status

Accepted — 2026-06-01

## Context

A concurrent GC cycles through four phases, each with different barrier
semantics and invariants:

1. **Idle** — steady state, no GC activity.
2. **Marking** — concurrent trace; mutators use SATB pre-barriers.
3. **RelocatePrep** — mark complete; preparing to compact.
4. **Relocating** — concurrent compaction; mutators use ref-upd barriers.
5. **Remap** — final remap pass.

The phase must be readable by every mutator thread with low overhead and
strong ordering guarantees. A stale phase read could lead to using the
wrong barrier variant.

## Decision

- **Storage**: `PhaseCell` is a 64-byte aligned `AtomicU8` to avoid false
  sharing with adjacent hot fields.
- **Encoding**: each phase is a distinct `u8` value (0-4).
- **Transitions**: `cas` for state-machine transitions; `store` for exit.
- **Memory ordering**: `Acquire` on load, `Release` on store, `AcqRel` on
  CAS. Sufficient for a single-writer / many-reader state machine.
- **Sentinel**: `Idle` is both the start and end state, allowing re-entry
  checks via `phase.cas(Idle, Marking)`.

## Alternatives Considered

| Approach          | Pros                       | Cons                            |
|-------------------|----------------------------|---------------------------------|
| Sequence lock     | Lock-free read             | Overhead on every phase check   |
| Global mutex      | Simple                     | Lock contention hot path        |
| Per-thread state  | Zero coordination          | Hard to reason about            |

## Consequences

- The phase check is a single atomic load (~1-2 cycles on x86).
- All other GC state is derived from the phase + epoch.
- `MarkPhase::as_str()` enables logging without formatting allocations.
- A new phase requires a bump in `MarkPhase` enum + table in `from_byte`.
