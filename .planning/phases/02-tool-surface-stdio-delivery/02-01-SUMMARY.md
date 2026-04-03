---
phase: 02-tool-surface-stdio-delivery
plan: 01
subsystem: api
tags: [rust, mcp, rmcp, schemas]

# Dependency graph
requires:
  - phase: 01-mcp-crate-interface-boundary
    provides: docracy-mcp bootstrap + operations + error mapping boundary
provides:
  - rmcp tool surface for init/create/read/query/update
  - stable schema-visible tool list
  - contract tests preventing tool/schema drift
affects: [phase-02-stdio-binary, phase-03-streamable-http]

# Tech tracking
key-files:
  created:
    - crates/mcp/src/tools.rs
    - crates/mcp/tests/tool_contract.rs
    - crates/mcp/README.md
  modified:
    - crates/mcp/src/error.rs
    - crates/mcp/src/lib.rs
    - crates/mcp/Cargo.toml
    - Cargo.lock

requirements-completed: [TOOL-01, TOOL-02, TOOL-03, TOOL-04, TST-01]

# Metrics
completed: 2026-04-06
---

# Phase 02 Plan 01: rmcp tools + schemas + error contract + tests Summary

Implemented an rmcp-based tool surface in `docracy-mcp` exposing **init/create/read/query/update** with machine-readable schemas, plus contract tests that prevent silent tool/schema drift.

## What Happened With The Hanging Agent

The executor got stuck running `cargo test -p docracy-mcp --test tool_contract` because the original tests deadlocked during the MCP initialization handshake.

Fix: the tests now start server/client concurrently (handshake-safe) and run inside a `tokio::task::LocalSet` (rmcp uses `spawn_local` in this workspace).

## Task Commits

1. **Task 1: rmcp tool handlers + schemas + error contract**
   - `2b4e127` feat(02-01): add rmcp tool router for core operations

2. **Task 2: contract tests (tool list + schema + error payload)**
   - `4de632c` test(02-01): add rmcp tool contract tests without deadlocks

3. **Task 3: tool contract documentation**
   - `05ffa58` docs(02-01): document MCP tool contract

## Verification

- `cargo test -p docracy-mcp` ✅

## Self-Check: PASSED

- FOUND: .planning/phases/02-tool-surface-stdio-delivery/02-01-SUMMARY.md
- FOUND commits: 2b4e127, 4de632c, 05ffa58
