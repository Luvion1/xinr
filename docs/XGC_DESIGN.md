# XGC Design Specification

## Overview

XGC (Xin Garbage Collector) is the automatic memory manager for the Xin
language. It is a **region-based, concurrent, ZGC-inspired** collector
with colored pointers and SATB/ref-update barriers.

## Goals

- **Low latency**: pause times bounded by GC phase, not heap size.
- **No stop-the-world**: marking and relocation run concurrently with
  mutator threads.
- **`no_std` capable**: runtime works on bare metal; XGC itself requires
  `alloc`.
- **Zero-panic library**: all failure modes return `Result<T, RuntimeError>`.

## Non-Goals

- **Generational collection**: deferred to a future iteration.
- **Real-time guarantees**: bounded worst-case pause is a research topic.
- **Compact (32-bit) targets**: requires side-table for colored pointers.
- **Xin standard library**: explicitly out of scope.

## Module Map

```
xinr/src/xgc/
├── mod.rs                 # Public exports
├── region/                # Heap layout
│   ├── mod.rs             # Region, REGION_SIZE
│   ├── table.rs           # RegionTable (allocator)
│   ├── bitmap.rs          # MarkBitmap (per-region liveness)
│   └── descriptor.rs      # RegionDescriptor (region + bitmap pair)
├── colored/               # Pointer tagging
│   ├── mod.rs             # Color enum
│   ├── mask.rs            # bit-level helpers (COLOR_MASK, paint, strip)
│   └── ptr.rs             # ColoredPtr wrapper
├── barrier/               # Mutator-side hooks
│   ├── mod.rs             # Re-exports
│   ├── load.rs            # LoadBarrier trait
│   ├── mark_state.rs      # MarkEpoch counter
│   ├── satb.rs            # SatbBuffer (pre-write barrier)
│   └── ref_update.rs      # RefUpdateBuffer (relocation phase)
├── mark/                  # Mark phase
│   ├── mod.rs             # Marker state machine
│   ├── phase.rs           # MarkPhase, PhaseCell
│   └── worklist.rs        # Worklist (LIFO)
├── relocate/              # Compaction
│   ├── mod.rs             # Relocator
│   ├── forward.rs         # Forward word, FORWARD_MAGIC
│   └── table.rs           # RelocTable (old→new map)
└── heap/                  # Heap subsystem
    ├── mod.rs             # Submodule re-exports
    ├── xgc.rs             # Xgc main handle
    ├── alloc.rs           # RegionAllocator
    └── sweep.rs           # Sweep functions
```

## Phase State Machine

```
   ┌─────────┐
   │  Idle   │◀──────────────┐
   └────┬────┘               │
        │ cas(Idle, Marking) │
        ▼                    │
   ┌─────────┐               │
   │ Marking │──finish──▶    │
   └─────────┘               │
        │                    │
        ▼                    │
   ┌───────────────┐         │
   │ RelocatePrep  │         │
   └───────┬───────┘         │
           ▼                 │
   ┌─────────────┐           │
   │ Relocating  │──finish──▶│
   └─────────────┘           │
           │                 │
           ▼                 │
   ┌─────────┐               │
   │  Remap  │────────────────┘
   └─────────┘
```

## Pointer Encoding (64-bit)

```
 63            3 2  0
┌──────────────┬─────┐
│   address    │meta │
└──────────────┴─────┘
   61 bits        3 bits
   address       0,1: color (White/Grey/Black)
                 2:   relocation flag
```

## Cycle Sequence

1. **Mark**:
   - `Xgc::begin_mark()` → phase = Marking, epoch += 1.
   - Push roots (stack, registers, statics) onto worklist.
   - GC thread pops grey slots, scans children, marks them grey.
   - When worklist is empty, all reachable slots are black.
2. **Sweep**:
   - For each region, walk the mark bitmap. Unmarked slots are released.
   - Region `used` is reset to 0.
3. **Relocate**:
   - `Xgc::begin_relocate()` → phase = Relocating.
   - For each live slot, copy to a fresh region.
   - Install forward pointer at the old address.
   - Process the ref-update buffer to fix dangling mutator writes.
4. **Remap**:
   - Process the SATB buffer (drain remaining snapshot entries).
   - `Xgc::finish_relocate()` → phase = Idle.

## Concurrency

- **Mutator threads** execute reads/writes as normal.
- **Read barrier** is on the slow path: only triggered when a pointer's
  color is non-white or the relocation bit is set.
- **Write barrier** is always emitted by the compiler; cost is one atomic
  load + buffer push.
- **GC thread** runs concurrently, advancing the worklist and relocator.

## Memory Budget

- Region size: 1 MiB.
- Bitmap per region: 16 KiB (REGION_SIZE / 8 / 8 = 16 KiB).
- Worklist: 4096 entries × 8 bytes = 32 KiB.
- SATB buffer: 256 entries × 8 bytes = 2 KiB per thread.
- Ref-update buffer: 128 entries × 16 bytes = 2 KiB per thread.

For a 1 GiB heap (1024 regions):
- Region table: 1024 × sizeof(Region) ≈ 32 KiB.
- Bitmaps: 1024 × 16 KiB = 16 MiB.

## Test Coverage

- 24 unit tests covering region, bitmap, worklist, phase, barriers, mark
  cycle, relocation cycle, and end-to-end Xgc lifecycle.
- All tests gated on `feature = "alloc"`.
- See `src/tests/` for the full suite.

## Status

| Component          | Status     |
|--------------------|------------|
| Region + bitmap    | ✅ implemented + tested |
| Colored pointer    | ✅ implemented + tested |
| Mark phase         | ✅ implemented + tested |
| Relocation         | ✅ implemented + tested |
| Barriers (SATB)    | ✅ implemented + tested |
| Barriers (ref-upd) | ✅ implemented + tested |
| Concurrent mark    | ⏳ skeleton (single-thread) |
| Full GC thread     | ⏳ deferred (not in scope of this iteration) |
| 32-bit side table  | ⏳ deferred |
| Generational       | ❌ future ADR |
