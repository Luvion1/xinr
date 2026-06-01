# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.1] - TBD

### Added
- `select_send_4` / `select_send_8` functions for non-blocking multi-channel sends
- `SendResult` struct for select_send return values
- `worker_pool_stress` tests: high-throughput channel + fiber stress tests
- `select_send_demo` example demonstrating sender-side select

### Documentation
- ADR 016: `try_join_all` design rationale