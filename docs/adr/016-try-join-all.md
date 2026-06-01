# 016: `try_join_all` Design

## Status

Accepted.

## Context

Structured concurrency requires waiting on multiple fibers/tasks simultaneously.
Users need to:
1. Wait for all spawned tasks to complete without blocking indefinitely
2. Optionally enforce a timeout on the collective join
3. Know how many tasks completed before the join concluded

## Decision

Introduce three primitives in `sync::join_all`:
- `try_join_all(tasks)` - returns when all tasks complete or any would block
- `try_join_all_with_timeout(tasks, ms)` - same but with timeout bound
- `count_ready(tasks)` - predicate counting how many tasks are ready to join

All functions are `no_std` compatible and use only `Option<T>` error handling.

## Consequences

- Enables deterministic coordination in test harnesses and worker pools
- Timeout variant allows bounded-latency shutdown
- Counter variant enables progress monitoring and early exit strategies