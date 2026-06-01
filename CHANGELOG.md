# Changelog

All notable changes to `xinr` are documented here. Versions follow SemVer.

## [Unreleased]

### Added
- **XGC** (24 subsystems):
  - region, colored, barrier, mark, relocate, heap, object, pressure,
    pin, diagnostics, finalize, card, worker, page, profile, hazard,
    cycle, budget, numa, tl, heuristics, log, slab, sched
- **Sync** (21 subsystems):
  - channel, oneshot, scope, barrier, semaphore, parking, fiber, condvar,
    blocking, rwlock, timer, waker, scheduler, select, spsc, metrics,
    cache_padded, **notify**, **waitgroup**, **timed_join**, **join_all**
- **`select!` macro** — ergonomic multi-channel polling (`sync::select!`)
- **Examples** (13): `basic_gc`, `channel_demo`, `full_pipeline`,
  `slab_demo`, `scheduler_demo`, `select_demo`, `pingpong`, `gc_worker`,
  `cache_demo`, `producer_consumer`, `shutdown_coord`, `gc_relocate_bench`,
  **`metrics_report`**
- **ADRs** (15): 001-015 documenting design decisions
- **Tests**: **272 passing** across 28 test files, 0 failed, 6 ignored (benchmarks)
- **ADRs 013, 014** — Memory-safe single-core config + benchmark macro
- **`.cargo/config.toml`** — pin build/test to 1 job and 1 thread
- **Documentation**: `README.md`, `CHANGELOG.md`, `XGC_DESIGN.md`,
  `docs/api/{xgc,sync}.md`, `docs/adr/001-011.md`

### Changed
- `Channel<T>` placeholder replaced with `BoundedChannel<T, N>` (typed capacity)
- `Fiber` placeholder replaced with full state machine + 4 KiB stack
- Fiber `park`/`unpark` relaxed to allow from any non-Finished state
- `Fiber::DEFAULT_STACK_SIZE` reduced to 4 KiB to fit in test stacks
- `ParkedThread` gained an explicit `active: bool` flag
- `RuntimeError` extended with `Disconnected`, `WouldBlock`, `Closed`
- `xgc` and `sync` are now `pub mod`
- `RwState` uses `UnsafeCell<i32>` so multiple read guards can coexist
- Guards require explicit `release()` instead of `Drop`
- `try_join_with_timeout` signature simplified (no longer takes `const W`)
- Removed ambiguous `pub use xgc::*; pub use sync::*` globs in `lib.rs`
  (use `crate::xgc::...` and `crate::sync::...` paths directly)
- `select!` macro now uses `expr` for the index and wraps arm in `Some($arm)`
- All `let_chains` (`if let ... &&`) collapses applied to clippy warnings
- `CycleQueue` gained a `Default` impl

### Added (continued)
- `SpscChannel<T, N>` — single-producer single-consumer
- `MpscChannel<T, N>` — multi-producer single-consumer
- `select_recv_4` / `select_recv_8` — poll multiple channels
- `Notify` — one-shot flag for cross-fiber signaling
- `WaitGroup` — count-down barrier for fan-in completion
- `try_join_with_timeout` — bounded wait join over `Oneshot + TimerWheel`
- `Metrics` counters and `CachePadded<T>` for hot-loop types
- `bench!` macro and `bench_test!` macro for micro-benchmarks (`xinr::bench`)
- **`bench_group!` macro** — run a group of variants with shared prefix
- **`RuntimeResult<T>`** alias + `From<()>` + `From<Infallible>` for `RuntimeError`
- **`RuntimeError::TimedOut`** + `is_transient()` + `is_fatal()` classification
- **`Metrics`** gains `live()`, `marks_per_cycle()`, `reset()`, `add(idx, delta)` (5 new tests)
- **Error tests** (6 unit tests in `error.rs`)
- **Slab/region integration tests** (2 new tests in `tests/xgc/tune/slab_region_integration.rs`)
- **2 new benchmark tests**: `bench_channel_group` (3 variants), `bench_xgc_mark_drain`
- **`join_all` subsystem** (alloc-gated) — `try_join_all`, `try_join_all_with_timeout`, `count_ready` (3 new tests)
- **heap_long_haul** stress test (1000 cycles, `#[ignore]`)
- **ADR 015** — Error classification (`is_transient`, `is_fatal`, `RuntimeResult`)
- **`time_it!` macro** — measure an expression and return `(ticks, result)`
- **13th example: `metrics_report`** — live counter inspection over 20 GC cycles
- `select!` macro test coverage (4 new tests in `tests/sync/access/async_/select_macro.rs`)
- `.github/workflows/ci.yml` — CI: fmt + clippy + build + test + examples
- **ADR 012** — Lint and Format Compliance

### Fixed
- Multiple-borrow conflicts in `RwLock` tests
- Duplicate `pub use` re-exports in `cycle/mod.rs`
- Stale module declarations in `slab/`, `fiber/`, `channel/`
- Stack overflow in scheduler tests
- Ownership issue in `select_send_4` (removed; recv-only select now)
- no-std build broken by `xgc` reference in `fiber/stack.rs`
  (`PAGE_SIZE` now defined locally)
- Ambiguous glob re-exports of `barrier` from both `xgc` and `sync`
- `crate::xgc::object::header` test path no longer relies on glob re-export
- 12 lib clippy warnings, 8 test warnings, 3 example warnings (all → 0)
- `fuzz/fuzz_target.rs` not in tree (not needed; static analysis suffices)
- `select_recv_4` test: `SelectResult` has `index`/`value` fields, not `Ready(2, 42)` variant
- `select!` macro: shadowed by `use crate::sync::select;` in tests
  (use absolute path `crate::select!{ ... }` or `use crate::select;`)
- Module inception `mod stress;` in `tests/xgc/stress/` (renamed to `run.rs`)
- `mod xgc;` defined twice in `tests/mod.rs` after `bench` add

## [0.1.0] — Initial placeholder

- Skeleton `lib.rs`, `error.rs`
- Empty `xgc/` and `sync/` module directories
