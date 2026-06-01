# ADR 015: Error Classification and Result Conventions

## Status

Accepted — 2026-06-01.

## Context

`xinr` was originally exposing a flat enum `RuntimeError` with no
classification. Callers had to pattern-match on individual variants
to know whether retrying was safe, whether the failure was fatal,
and how to surface it to the host. With the addition of the
`TimedOut` variant (for `try_join_with_timeout`) this became acute.

## Decision

Add three layers of classification to `RuntimeError`:

1. **Numerical code** — `code(self) -> usize` for interop with macro
   matching and host runtimes that only see integers.
2. **Category predicates**:
   - `is_transient(self) -> bool` — `WouldBlock | TimedOut`. Safe to
     retry; nothing about the program state is wrong.
   - `is_fatal(self) -> bool` — `OutOfMemory | StackOverflow`. The
     runtime cannot continue; the caller must abort.
3. **Result type alias** — `pub type RuntimeResult<T> = Result<T, RuntimeError>`
   so signatures are uniform across the codebase.
4. **Conversion impls** — `From<()>` and `From<Infallible>` map to
   `Ok` so trivial closures compile as `RuntimeResult`.

The 9 → 10 variant growth is justified: `TimedOut` is semantically
distinct from `WouldBlock` (it means "I waited and gave up", not
"I would have to wait"). The two are not interchangeable.

## Consequences

- All `?` propagation works uniformly across subsystems.
- `try_join_with_timeout` returns `TimedJoin<T>` (its own enum with
  `Ready | Timeout`) because the user wants the value-or-Timeout
  semantic, not a `Result`.
- The `Metrics` struct gains `add(idx, delta)` so callers can route
  counter increments through a single dispatch site, keeping the
  metrics surface uniform with the error surface.
- The bench harness gains a `bench_group!` macro that records each
  variant under a shared prefix (`<prefix>::<variant>`) and returns
  the total elapsed ticks. This complements the new classification
  system by providing a uniform way to time multi-variant flows.
- 6 new error unit tests live in `error.rs`. They are `alloc`-gated
  to keep the no_std lib size small (the test needs `format!`).

## Alternatives considered

- **Returning `anyhow::Error` from `alloc`-only paths** — bloats the
  type, no benefit when the variants are known. Rejected.
- **Three-level error hierarchy (sub-enums)** — over-engineered for
  10 variants. Rejected in favor of predicates.
- **Per-subsystem error types** — would require repeated `From`
  impls and a top-level `enum RuntimeError { Xgc(XgcError), ... }`.
  Not worth the indirection for the size of the current API.
