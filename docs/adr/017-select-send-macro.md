# 017: select_send! Macro Design

## Status

Accepted.

## Context

The `select_recv_4` and `select_recv_8` functions provided non-blocking receive
across multiple channels. Users needed a corresponding send-side primitive for
symmetric select operations when producers compete for the first available consumer.

## Decision

Add `select_send!` macro orthogonal to `select!` macro. This follows the same
pattern as the function variants (`select_recv_4` vs `select_send_4`) and keeps
the API surface regular.

## Consequences

- Producers can now select on multiple channels for the first with room
- Symmetric API: `select!` for receive, `select_send!` for send
- Both macros return `Option<T>` where T is the arm expression value