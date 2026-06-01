# ADR-007: `no_std` Design with Optional `alloc`

## Status

Accepted — 2026-06-01

## Context

`xinr` is a runtime library. It must work in two deployment contexts:

1. **Hosted**: a normal OS process with `std` available (developer machine,
   server, container).
2. **Bare metal**: embedded, kernel modules, or `no_std` targets where only
   `core` is available.

The XGC subsystem itself depends on `alloc` (region table, bitmaps, worklists
are heap-allocated). The error type and `core`-only utilities must work
in `no_std`.

## Decision

- **Default features**: empty. Caller must opt in to `std` or `alloc`.
- **`std` feature**: enables `alloc` plus `std::error::Error` impl.
- **`alloc` feature**: enables heap allocation; required for XGC.
- **`no-std` feature**: marker for explicit bare-metal builds.
- **Module gating**:
  - `error.rs`, `diagnostics.rs`, `sync/` always compiled.
  - `xgc/` only compiled with `alloc` or `std`.
- **No `std` in `error.rs`**: manual `core::fmt::Display` impl.
  - `std::error::Error` only impl'd under `#[cfg(feature = "std")]`.

## Alternatives Considered

| Approach          | Pros                   | Cons                                |
|-------------------|------------------------|-------------------------------------|
| `std`-only        | Simple                 | Cannot target bare metal            |
| `no_std`-only     | Maximum portability    | Limits hosted ergonomics            |
| Two crates        | Clear separation       | Maintenance overhead                |

## Consequences

- The build matrix covers 4 configurations: no-std, alloc, std, all-features.
- `std::error::Error` impl is opt-in; no_std consumers get a custom
  `RuntimeError` with `description()` and `code()` methods.
- The XGC API is unavailable without `alloc`; the rest of the runtime
  (sync primitives, diagnostics) is always usable.
- Feature flags are documented in `docs/README.md` and `Cargo.toml`.
