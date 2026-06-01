# XGC API Reference

The `xinr::xgc` module provides a ZGC-inspired concurrent garbage
collector. It is gated behind the `alloc` feature.

## Heap

### `Xgc`

Main XGC handle. Manages regions, marking, and relocation.

```rust
use xinr::xgc::Xgc;

let mut gc = Xgc::new(64).unwrap();  // 64 regions
gc.init().unwrap();
gc.begin_mark().unwrap();
// push roots, mark, etc.
gc.finish_mark();
gc.shutdown().unwrap();
```

## Region

### `Region`

A 1 MiB region of the heap. Holds a bitmap of mark bits and a relocation
forward pointer.

### `RegionTable`

Hash table mapping region addresses to `Region` records.

## Colored Pointers

### `ColoredPtr`

A pointer with 2 color bits embedded (ZGC-style).

```rust
use xinr::xgc::colored::{Color, ColoredPtr};
let p = ColoredPtr::new(0x1000, Color::White);
assert_eq!(p.color(), Color::White);
```

### `Color`

`White`, `Grey`, `Black`.

## Barriers

### `LoadBarrier`

Trait for objects that want to participate in the load barrier. The XGC
worker calls `load()` on the object during concurrent marking.

### `SatbBuffer`

Pre-write barrier: snapshot an old reference value before the mutator
overwrites it.

### `RefUpdateBuffer`

Post-write barrier: record a new reference value for the next mark cycle.

## Mark

### `Worklist`

Concurrent worklist of `ColoredPtr` to be scanned. Capacity 1024.

### `MarkPhase`

Phase cell: `Idle`, `Marking`, `Relocating`, `Remapping`, `Sweeping`.

## Relocate

### `ForwardTable`

Records new addresses for objects moved during compaction.

### `Relocator`

Performs the relocation: walks the forward table, copies bytes, updates
references.

## Object

### `ObjectHeader`

Magic + size + hash + age. 12 bytes, prepended to every object.

### `SizeClass`

`S16`, `S32`, `S64`, `S128`, `S256`, `S512`, `Large`.

### `Trace` / `Visitor`

Traits for objects that have references to trace.

## Memory Management

### `SlabAllocator`

Power-of-two fixed-size slab allocator. One slab per `SlabSize`.

### `WorkerPool`

Work-stealing worker pool. Up to 8 workers.

### `WorkDeque`

Per-worker double-ended task queue. LIFO locally, FIFO for steals.

## Pressure & Heuristics

### `PressureMeter`

Tracks allocation vs. collection to decide when to trigger a cycle.

### `CollectionTrigger`

Combines high-watermark, promotion count, and allocation count into a
`Trigger` decision.

### `HeapSizing`

High/low watermarks and grow/shrink factors for the heap.

## NUMA

### `NumaTopology`

Per-node capacity tracking. Use `most_free()` for placement.

### `NodeId`

8-bit node id. `LOCAL = 0`, `ANY = u8::MAX`.

## Thread-Local

### `Tlb`

Thread-local allocation buffer. 8 slots.

### `ThreadCtx`

Per-thread context: id, TLB, allocs, hits, misses, hit_rate.

## Budget

### `CycleBudget`

Per-phase millisecond caps.

### `BudgetTracker`

Tracks elapsed time and answers "is the mark budget exceeded?" etc.

### `Instant`

Monotonic millisecond counter.

## Log

### `EventLog`

256-entry ring buffer of `GcEvent`s.

### `EventKind`

`Init`, `MarkStart`, `MarkEnd`, `RelocateStart`, `RelocateEnd`,
`SweepStart`, `SweepEnd`, `Alloc`, `Promote`, `Evict`, `Cycle`,
`Trigger`, `Error`.

## Finalize

### `FinalizationQueue`

Drops user-supplied finalizers in the GC thread.

### `WeakTable`

Weak references that auto-upgrade to strong when the referent is alive.

## Hazard

### `HazardRecord` / `HazardSlot`

Lock-free hazard pointer records. A retiring thread must wait until all
readers have dropped their references.

## Cycle

### `CycleDetector`

Identifies and reclaims unreachable cycles. Submits candidates, runs
passes, optionally triggers a full DFS.

## Card

### `CardTable`

Cards track which pages contain references into the young generation.
Updated by write barriers; scanned during minor GC.

## Page

### `PageTable`

Per-page metadata: state (`Free`, `Allocated`, `Marked`, `Swept`),
allocated count, generation.

## Profile

### `SiteStats` / `SiteId`

Allocation site profiling: tracks which code sites allocate the most.

## Error Variants

XGC-related error variants in `RuntimeError`:

- `OutOfMemory` — region allocation failed
- `AlreadyInitialized` — Xgc::init called twice
- `NotInitialized` — operation requires init
- `InvalidRegion` — region index out of bounds
