# ADR-003: Colored Pointer Encoding

## Status

Accepted — 2026-06-01

## Context

Concurrent GC must track liveness without freezing mutator threads. A
traditional mark-bit stored next to the object works for STW collectors,
but concurrent collectors need the bit to be readable atomically with the
pointer read, ideally in a single cache line.

ZGC pioneered "colored pointers": the low 2 bits of the aligned pointer
encode the mark color directly. The mutator's read barrier masks off the
color bits to recover the true address.

## Decision

- **Encoding**: low 2 bits = color; bit 2 = relocation flag.
- **Colors**: `White=0b00`, `Grey=0b01`, `Black=0b10`.
- **Relocation flag**: set when the object has been forwarded during
  the current cycle.
- **Address recovery**: `ptr & !META_MASK` (3-bit mask).
- **Type**: `ColoredPtr(usize)` is `#[repr(transparent)]`.

## Alternatives Considered

| Approach                | Pros                          | Cons                                 |
|-------------------------|-------------------------------|--------------------------------------|
| Side-table (colored map) | Works on 32-bit pointers     | Extra cache miss on every load       |
| Object header mark bit  | Simple                        | Header word read on every load       |
| Per-region color table  | Compact                       | Indirect through region              |

## Consequences

- 64-bit objects must be aligned to 8 bytes (3 free low bits).
- Object size must be a multiple of 8 bytes.
- `ColoredPtr` is a thin wrapper; pass-by-value is zero-cost.
- Pointer arithmetic must go through `ColoredPtr::addr()` to strip metadata.
- 32-bit platforms: side-table fallback is required (future work).
