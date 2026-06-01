# Xin runtime — contributing guide

xinr is the Xin language's runtime library. It is `no_std` by default
with an `alloc` feature that gates the XGC subsystem. All contributions
must keep both `cargo build --no-default-features` and
`cargo build --features alloc` clean.

## Build & test

```sh
# Format
cargo fmt --all

# Lint (both configurations, treat warnings as errors)
cargo clippy --features alloc --all-targets -- -D warnings
cargo clippy --no-default-features --lib --tests -- -D warnings

# Run tests
cargo test --features alloc           # full suite
cargo test --features alloc -- --ignored   # benchmarks + long-haul

# Run all examples
for ex in basic_gc channel_demo full_pipeline slab_demo scheduler_demo \
          select_demo pingpong gc_worker cache_demo producer_consumer \
          shutdown_coord gc_relocate_bench metrics_report; do
    cargo run --features alloc --example "$ex"
done
```

## Coding conventions

- Max 200 SLOC per file.
- Max 5 files per folder (deeper nesting for more).
- Min 10% of a file's lines are doc comments on public items.
- No `unwrap()` in library code; tests may use it.
- No unsafe outside `core::sync::atomic` and the `bench::_timestamp` helper.
- `let_chains` (Rust 2024) is encouraged for `if let ... && ...`.

## Folder layout

```
xinr/
├── src/
│   ├── lib.rs           # crate root, #![no_std]
│   ├── error.rs         # RuntimeError + RuntimeResult
│   ├── bench/           # bench! / bench_group! / bench_test! / time_it!
│   ├── sync/            # 21 sync subsystems
│   ├── xgc/             # 24 XGC subsystems (alloc-gated)
│   └── tests/           # all integration + unit tests
├── examples/            # 13 runnable examples
├── docs/
│   ├── adr/             # 15 architecture decision records
│   ├── api/             # API reference
│   └── XGC_DESIGN.md
├── .github/workflows/   # CI
├── .cargo/config.toml   # jobs=1, RUST_TEST_THREADS=1
└── Cargo.toml
```

## Submitting changes

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/my-change`.
3. Make your change; ensure `cargo fmt` is clean and clippy passes.
4. Add tests covering the new behavior in `src/tests/`.
5. If you add a new public type or change a public API, add an ADR in
   `docs/adr/NNN-short-title.md` and reference it from the change.
6. Push the branch and open a pull request against `main`.

## Releasing

xinr follows SemVer. Releases are cut by a maintainer using
`gh release create vX.Y.Z` from the `main` branch.
