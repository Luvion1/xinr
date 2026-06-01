# ADR 014: Benchmark Macro and Stable Test Harness

## Status

Accepted — 2026-06-01.

## Context

Xin runtime needs reproducible micro-benchmarks for hot paths
(channel send/recv, GC barrier overhead, hazard record/publish).
Two constraints:

1. Stable Rust does not support `#[bench]`. Nightly-only benches
   block CI.
2. Micro-benchmarks must not run during normal `cargo test` —
   they are slow, noisy, and destabilise the CI green/red signal.

## Decision

Provide two macros in the `bench` module:

- `bench!($name:literal, $body:expr)` — wraps an expression in a
  timestamp measurement. On stable, returns the result. On nightly
  with `#[bench]` it can be re-implemented via a custom build
  feature.
- `bench_test!($name:ident, $body:expr)` — expands to
  `#[test] #[ignore = "benchmark; run with --ignored"]` so the
  benchmark only runs when explicitly requested via
  `cargo test -- --ignored`.

The `_timestamp()` helper is a no-frills `static mut` counter.
It is documented as "useful only for relative comparisons" and is
not a wall-clock source. ADR 013 pins test threads to 1 so the
counter does not race.

A user-supplied `bench-host` feature may later override
`_record` to push to a host metrics sink.

## Consequences

- 3 bench files exist as `#[ignore]`-d tests: `hotpath.rs` in
  `src/tests/bench/`. They count as `3 ignored` in the test report.
- Nightly migration path is straightforward: rewrite `bench!` to
  emit `#[bench]` under `cfg(bench)`.
- The macros are `#[macro_export]`ed at the crate root, accessible
  as `xinr::bench!`.
- The `static mut` is acceptable because (a) ADR 013 pins threads
  to 1, (b) the value is purely informational, and (c) `no_std`
  environments may not have `Instant`.

## Alternatives considered

- **`criterion` external crate** — too heavy, depends on `std`,
  pulls in 14 transitive deps. Rejected.
- **Custom `feature = "bench"` gate** — forces every consumer to
  toggle the feature. Rejected in favor of `#[ignore]`.
- **`#![feature(test)]` behind a cfg** — locks users to nightly.
  Rejected; the macro leaves a nightly migration door open
  without requiring it today.
