# ADR-008: Latency, Placement, Observability, Adaptive Heuristics, Fast Paths

## Status

Accepted, 2026-06-01.

## Context

The first seven ADRs established the core XGC pipeline (region, color, barrier,
mark, relocate, heap) and the supporting subsystems (object, pressure, pin,
diagnostics, finalize, card, worker, page, profile, hazard, cycle). The
runtime was functionally complete but lacked the operational concerns that
make a GC production-grade: bounded pause time, NUMA awareness, per-thread
fast paths, adaptive collection decisions, and a timeline log for debugging.

## Decision

Add five more XGC subsystems:

### 1. Budget (`xgc::budget`)

Soft real-time cycle budgets. Each phase has a time limit; the GC checks the
budget at safe points and yields when exceeded.

- `CycleBudget` — per-phase millisecond caps.
- `BudgetTracker` — measures elapsed time and answers "is the mark budget
  exceeded?" etc.
- `budget::clock::Instant` — monotonic clock abstraction.

Default: 5/2/5/1 ms (mark/sweep/relocate/remap). Generous and unbounded
profiles also provided.

### 2. NUMA (`xgc::numa`)

Per-node capacity tracking for placement decisions.

- `NodeId` — opaque 8-bit node id with `LOCAL` and `ANY` sentinels.
- `NodeCapacity` — total, used, region count.
- `NumaTopology` — up to 16 nodes, `most_free()`, `reserve()`, `release()`.

`Xgc` can use this to place new regions on the node with the most free
space, or to keep allocations local to the thread's node.

### 3. TL (`xgc::tl`)

Thread-local allocation buffer to bypass heap locking on the common path.

- `Tlb` — fixed-capacity (8) stack of `ColoredPtr` for hot objects.
- `ThreadCtx` — thread id, TLB, allocs, hits, misses; reports `hit_rate()`.

Hit rate is monitored; if it drops below 50% the worker should re-fill its
buffer from the heap before continuing to allocate.

### 4. Heuristics (`xgc::heuristics`)

Adaptive collection decisions.

- `HeapSizing` — high/low watermarks, grow/shrink factors. Conservative
  (0.8/0.3) and aggressive (0.6/0.2) profiles.
- `CollectionTrigger` — decides `Trigger::{None, Full, High, Promotion, Allocation}`.

Triggers compose: if the heap is full, `Full` wins; otherwise, watermark,
promotion, or allocation thresholds.

### 5. Log (`xgc::log`)

Timeline ring buffer for post-mortem analysis.

- `EventKind` — `Init`, `MarkStart`, `MarkEnd`, `RelocateStart`, `RelocateEnd`,
  `SweepStart`, `SweepEnd`, `Alloc`, `Promote`, `Evict`, `Cycle`, `Trigger`,
  `Error`.
- `EventLog` — 256-entry ring; `total()` (ever), `len()` (current), `iter_chrono()`
  for chronological iteration.

## Consequences

Positive:
- Soft real-time guarantee via per-phase budget.
- NUMA-aware placement reduces cross-socket traffic.
- TLB cuts lock contention on the common allocation path.
- Heuristics adapt to workload, reducing both over- and under-collection.
- Log makes production debugging tractable.

Negative:
- More state to maintain; budget overruns are silent (GC just yields).
- NUMA requires the OS to report topology (caller's responsibility).
- TLB may hold references to reclaimed objects — must be cleared at safe points.
- Heuristics are conservative; pathological workloads may still thrash.

## Alternatives Considered

- **Hard-realtime deadlines**: rejected — incompatible with concurrent collection.
- **Work-stealing scheduler**: deferred — current worker pool is single-mutator,
  work-stealing needs a true thread pool.
- **Always-shrink heap**: rejected — causes thrash on bursty workloads.
- **Unbounded log**: rejected — needs to be O(1) memory regardless of runtime.
