# ADR 013: Memory-Safe Single-Core Build Configuration

## Status

Accepted — 2026-06-01.

## Context

Xin runtime compiles inside sandboxed environments with tight memory
budgets (≤512 MB resident, 1 vCPU, no swap). Default Cargo parallelism
(∞) immediately exhausts memory on `cargo test`, producing silent
OOM-kill and flaky CI.

`no_std` and `alloc` feature configurations both trigger the problem
because the same toolchain runs both in the same CI job.

## Decision

Add `.cargo/config.toml` at the crate root that pins the build/test
pipeline to a single job and a single test thread:

```toml
[build]
jobs = 1

[test]
jobs = 1

[env]
RUST_TEST_THREADS = "1"
```

- `[build] jobs = 1` — runs `rustc` instances sequentially. Without
  this, cargo spawns N rustc's in parallel, each holding 1–2 GB
  peak RSS for XGC codegen.
- `[test] jobs = 1` — runs test binaries one at a time, halving
  peak RSS vs default.
- `RUST_TEST_THREADS = "1"` — within a single test binary, runs
  tests serially so the bench-internal `static mut` counter does
  not race.
- Override per-invocation when more memory is available:
  `cargo build -j 4`.

## Consequences

- `cargo test` is now 2–3× slower (sequential compile of 156 files).
- Memory peak is bounded to a single rustc + single test binary at
  a time, dropping from ≈4 GB to ≈1.2 GB.
- The `.cargo/config.toml` is local to the crate and does not
  affect the workspace or other crates.
- The file is committed to the repo so the same config is shared
  across CI and local developer sandboxes.
- The `static mut` counter in `bench::_timestamp` becomes
  single-threaded by construction, eliminating the prior
  `unsafe` race.

## Alternatives considered

- **Workspace-level `config.toml`** — applies to all crates, even
  those that genuinely need parallelism. Rejected for being too
  broad.
- **`-j 1` in CI script only** — drifts from local; developers
  hit OOM locally first. Rejected.
- **Disabling parallel codegen in `profile.release`** — does not
  fix debug build OOM. Rejected.
- **No config file, document `cargo -j 1` in README** — easy to
  forget. Rejected in favor of repository-pinned config.
