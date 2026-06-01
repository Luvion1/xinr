# ADR 012 — Lint and Format Compliance

## Status
Accepted, 2026-06-01.

## Context
The `xinr` crate had several style issues accumulated during the
"build-test-fix" cycle:

- 12 lib warnings from `cargo clippy` (`if` collapses, `div_ceil` reimpl,
  redundant `if let` over `Some`, missing `Default` impl, etc.).
- 8 test warnings from constant-true asserts, unnecessary `&mut` references,
  module-inception (`mod stress;` inside `stress/`).
- 3 example warnings from useless conversions and `Some`-with-`ok()` patterns.
- `cargo fmt --check` flagged several indentation issues in
  recently-rewritten test files.

We treat `-D warnings` on clippy as a hard CI gate, and the `no_std`
build must also be lint-clean.

## Decision
1. **Enforce `cargo clippy --all-targets -- -D warnings`** for both
   `--features alloc` and `--no-default-features --lib --tests`.
2. **Run `cargo fmt --check`** as a separate CI step.
3. **Use `let_chains`** (Rust 2024) to collapse nested `if let` / `if`
   patterns where the compiler can prove both arms mutually exclusive.
4. **Prefer `Option::iter().filter_map(|x| *x)`** over `flatten()` when
   working with fixed-size arrays of `Option`, because
   `Option<T>` does not implement `IntoIterator` (only `&[T]` does).
5. **Mark intentionally-unread struct fields with `#[allow(dead_code)]`**
   at the struct level (e.g. `SizePool`, `GcWorker`) rather than `_field`
   renaming, which would obscure the intent.
6. **Rename child modules to avoid inception**:
   `src/tests/xgc/stress/stress.rs` → `run.rs` (parent already named
   `stress`).
7. **`const { assert!(..) }` block** for compile-time checks instead of
   runtime `assert!` on constant expressions.

## Consequences
- `cargo clippy` returns 0 warnings on `--features alloc --all-targets`
  and on `--no-default-features --lib --tests`.
- `cargo fmt --check` returns 0 diffs.
- 257/257 tests pass; 11/11 examples run.
- The `let_chains` syntax is a Rust 2024 feature; the crate's
  `Cargo.toml` uses `edition = "2024"`.
- `RUSTFLAGS: -D warnings` in CI catches any regression.

## References
- `src/lib.rs`: `#![no_std]`, `#![allow(clippy::module_name_repetitions, clippy::missing_docs_in_private_items)]`.
- `.github/workflows/ci.yml`: lint, build, test, and example matrix.
- AGENTS.md §11.1: max 200 SLOC per file, max 5 files per folder,
  min 20% documentation ratio.
