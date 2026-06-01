# xinr — Xin Runtime

## Overview

`xinr` adalah runtime library murni untuk bahasa Xin. Fokusnya adalah **XGC (Xin Garbage Collector)** dan primitif sinkronisasi. `xinr` **tidak menyediakan standard library Xin** — itu domain lain.

## Goals

- XGC: Region-based concurrent GC (ZGC-inspired) untuk low-latency.
- Sync primitives: Fiber + channel untuk structured concurrency.
- `no_std` first: core runtime bekerja tanpa OS, dengan optional `alloc`.
- Zero-panic library code: semua kegagalan diketik kuat (`Result<T, E>`).

## Non-Goals

- Xin standard library (`std.Collections`, `std.IO`, dst).
- Package manager, build system.
- Language frontend (milik `xinc`).

## Feature Flags

| Flag     | Meaning                             |
|----------|-------------------------------------|
| `std`    | Default. OS integration, `alloc`.   |
| `alloc`  | Heap only, no OS I/O.               |
| `no-std` | Bare metal, completely standalone.  |

## Structure

```
xinr/src/
├── lib.rs              # crate root, feature gates
├── core/               # platform shim, alloc re-exports
├── xgc/                # XGC (heap, region, colored, barrier)
│   ├── mod.rs
│   ├── heap/xgc.rs     # public Xgc handle
│   ├── region/mod.rs   # region metadata
│   ├── colored/mod.rs  # color encoding
│   └── barrier/mod.rs  # load barrier + mark state
└── sync/               # structured concurrency
    ├── fiber/
    └── channel/
```

## Status

Early implementation. Docs-first approach: design recorded before code.
