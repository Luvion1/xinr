# Security policy

## Supported versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |

## Reporting a vulnerability

If you discover a security vulnerability in `xinr`, please report it
privately by opening a GitHub issue with the `security` label, or by
contacting the maintainers directly via the GitHub repository.

Please do **not** disclose the vulnerability publicly until a fix has
been released.

## Soundness

`xinr` is intended to be sound under both `--no-default-features` and
`--features alloc`. The CI pipeline enforces this with
`cargo clippy --all-targets -- -D warnings` for both configurations.

Unsafe code is restricted to:
- `core::sync::atomic` operations (always with explicit `Ordering`).
- The `bench::_timestamp` coarse counter (single-threaded by ADR 013).
- The `ParkingLot` permit/wake primitive.

All other code is `#[forbid(unsafe_code)]` in spirit; if you need to
add `unsafe`, please open a discussion issue first.
