# ADR-002: Region-Based Heap Layout

## Status

Accepted — 2026-06-01

## Context

XGC must minimize GC pause times while supporting large heaps (>= 1 TiB).
A monolithic heap with a single mark bitmap forces the GC thread to scan
all live data even when only a small fraction is garbage. Generational
approaches add young/old invariants and remember-set maintenance.

ZGC addresses this by partitioning the heap into fixed-size regions. Each
region carries its own metadata (occupancy, mark bitmap) and is the unit
of allocation, sweep, and relocation.

## Decision

- **Region size**: 1 MiB (constant `REGION_SIZE`).
- **Layout**: contiguous array of `Region` descriptors (`RegionTable`).
- **Mark bitmap**: 1 bit per 8-byte object slot = 16 KiB per region.
- **Allocation**: bump pointer within a region; spill to next region on full.
- **Sweep**: walk the region table linearly; reset used bytes and bitmap.
- **Relocate**: move live slots from old region to fresh one; leave
  forwarding pointer at the old address.

## Alternatives Considered

| Approach          | Pros                   | Cons                                      |
|-------------------|------------------------|-------------------------------------------|
| Monolithic heap   | Simple                 | Pause scales with heap size               |
| Generational      | Better nursery locality | Remember-set maintenance, complex invariants |
| Slab allocator    | Good for fixed-size     | Poor for variable-sized heap objects     |

## Consequences

- Pointer arithmetic between regions is straightforward (region base + offset).
- Region table is fixed-size; allocatable heap = `num_regions * REGION_SIZE`.
- Region metadata is **pinned** (never moved by GC).
- 64-bit pointer encoding uses low 2 bits for color and bit 2 for relocation.
- 32-bit targets require a side-table fallback (deferred to a future ADR).
