# ADR-001: XGC Architecture — ZGC-inspired Concurrent Region-based GC

## Status

Accepted — 2026-06-01

## Context

Xin adalah bahasa systems programming dengan automatic memory management.
Target: low-latency applications tanpa stop-the-world pauses.

Opsi GC yang dievaluasi:
1. **Stop-the-world mark-compact** — sederhana, tapi pause = full heap scan. Tidak cocok untuk low-latency.
2. **Non-generational concurrent tracing GC** (G1-style) — kompleksitas orde marking + SATB.
3. **ZGC-inspired design** — concurrent + region-based + colored pointers.

## Decision

Adopsi arsitektur ZGC:

- **Concurrent**: Marking, relocation, dan remapping berjalan secara concurrent dengan mutator threads.
- **Region-based**: Heap dibagi region (1 MiB default). Setiap region punya metadata: occupancy, live bitmap.
- **Colored pointers**: State dari setiap object di-encode di pointer (low 2–3 bits) atau side table.
- **Load (read) barriers**: Mutator thread menjalankan ~5-cycles barrier tiap load pointer agar tetap konsisten dengan GC concurrent.
- **No generasi** secara default: simple region layout tanpa young/old gen. Opt-in generasi mendatang.
- **No card tables / no remembered sets**: Region granularity mungkin di-encode di warna pointer untuk inter-region references.

## Alternatives Considered

| Approach     | Pause | Complexity | Notes                                    |
|--------------|-------|------------|------------------------------------------|
| STW mark-sweep | High     | Low         | Rejected — latency target tidak terpenuhi  |
| Generational | Med   | High       | Rejected — extra invariants untuk nursery |
| Reference counting | None | Low | Rejected — tidak aman untuk cycles, overhead write |

## Consequences

- Compiler (`xinc`) harus instrumentasikan load pointer dengan barrier call di setiap read heap slot.
- Runtime berisi single `Xgc` global instance selama proses.
- Portable targets (e.g. 32-bit) memerlukan side table untuk colored pointers (fallback).
- Region metadata tidak boleh di-GC'd (pinned).

## References

- [ZGC: Scalable Low-Latency Garbage Collector](https://openjdk.org/projects/zgc/) (OpenJDK)
- `docs/02-region-based-memory.md`
- `docs/03-colored-pointers.md`
- `docs/04-load-barriers.md`
