# ADR-006: Relocation and Forwarding Pointers

## Status

Accepted — 2026-06-01

## Context

After concurrent marking, the GC must compact live objects to reclaim
fragmented regions. Compaction moves objects from old addresses to fresh
ones; concurrent mutators holding old references must be served correctly.

ZGC handles this by:
1. Copying live objects to a new region.
2. Installing a **forwarding pointer** at the old address.
3. Mutators (via load barrier) follow the forward when they see a relocated
   pointer; the relocation table maps `old → new` for batch fixes.

## Decision

- **Forward word**: stored at the old address, points to the new address.
- **Magic word** `FORWARD_MAGIC` (= 0xFEED_F00D_CAFE_BABE) sentinel.
- **Relocation table**: sparse `RelocTable` of `(old, new)` pairs.
- **Relocator**: drives the relocation phase; tracks `moved` count.
- **Resolution**: `Relocator::resolve(old)` returns the new pointer or
  the original if not relocated.
- **Capacity**: `RelocTable` is fixed at 1024 entries (deferred → growable).

## Alternatives Considered

| Approach               | Pros                       | Cons                          |
|------------------------|----------------------------|-------------------------------|
| Brooks-style forwarding | Always one indirection    | Extra word per object         |
| Card-table compaction  | Less mutation              | More complex bookkeeping     |
| Mark-and-don't-sweep   | Simple                     | Fragmentation                |

## Consequences

- Old addresses remain valid until the next cycle; reads through the
  load barrier must check the relocation bit in the colored pointer.
- The relocation table is scanned by mutator threads; reads need to be
  atomic in the future (deferred → lock-free copy-on-write).
- Forwarding is one-shot per cycle; after `Remap`, the old address is
  considered dead.
