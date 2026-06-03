# xinr — Xin runtime

[![CI](https://github.com/Luvion1/xinr/actions/workflows/ci.yml/badge.svg)](https://github.com/Luvion1/xinr/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Luvion1/xinr)](https://github.com/Luvion1/xinr/releases)

`xinr` is a `no_std`-friendly runtime library for the Xin ecosystem.

- **XGC** — ZGC-inspired concurrent, region-based, colored-pointer GC (24 subsystems)
- **Sync** — structured concurrency primitives (21 subsystems)
- **Bench** — micro-benchmark macro

## Status

| Build | Status |
|---|---|
| `cargo check --no-default-features` | ✓ clean, 0 warnings |
| `cargo build --features alloc` | ✓ clean, 0 warnings |
| `cargo clippy --all-targets -- -D warnings` | ✓ 0 warnings (alloc + no_std) |
| `cargo fmt --check` | ✓ clean |
| `cargo test --features alloc` | ✓ **277 passed, 0 failed, 7 ignored** |
| 14 examples | ✓ all run |
| `.cargo/config.toml` | `jobs = 1` for build & test (memory-safe) |

## Examples

| Example | Subsystems |
|---|---|
| `basic_gc` | XGC core |
| `channel_demo` | channel, oneshot, scope, barrier, semaphore, parking, fiber |
| `full_pipeline` | XGC + scope + parking + fiber + RwLock + timer |
| `slab_demo` | SlabAllocator, SlabSize |
| `scheduler_demo` | Scheduler, parking, timer, waker, fiber |
| `select_demo` | SPSC, MPSC, select |
| `pingpong` | Fiber, channel |
| `gc_worker` | XGC + scope + log + work channel + timer + fiber |
| `cache_demo` | RwLock, parking, metrics, channel |
| `producer_consumer` | SPSC, parking, metrics, parking lot |
| `shutdown_coord` | WaitGroup, Notify, scope, oneshot, fiber |

## Features

- `default = []` — no_std, sync available, XGC disabled
- `no-std` — explicit alias
- `alloc` — enable XGC and all heap-backed subsystems
- `std` — standard library support (deferred)

## Quick start

```ignore
use xinr::xgc::Xgc;
use xinr::sync::channel::BoundedChannel;

let mut gc = Xgc::new(64).unwrap();
gc.init().unwrap();
gc.begin_mark().unwrap();

let mut ch: BoundedChannel<u32, 8> = BoundedChannel::new();
ch.try_send(42).unwrap();
let v = ch.try_recv().unwrap();

gc.finish_mark();
gc.shutdown().unwrap();
```

## Modules

### XGC (24)

region, colored, barrier, mark, relocate, heap, object, pressure,
pin, diagnostics, finalize, card, worker, page, profile, hazard,
cycle, budget, numa, tl, heuristics, log, slab, sched

### Sync (20)

channel, oneshot, scope, barrier, semaphore, parking, fiber, condvar,
blocking, rwlock, timer, waker, scheduler, select, spsc, metrics,
cache_padded, **notify**, **waitgroup**, **timed_join**

`select!` macro: `xinr::sync::select!` for polling 2-8 channels.

## Documentation

- `CHANGELOG.md` — release history
- `XGC_DESIGN.md` — high-level design
- `docs/api/xgc.md` — XGC API reference
- `docs/api/sync.md` — sync API reference
- `docs/adr/001-012` — 12 architecture decision records
- `.github/workflows/ci.yml` — CI pipeline (fmt + clippy + test + examples)

## License

(Insert project license here.)
