---
phase: 02-tool-surface-stdio-delivery
plan: 02
subsystem: api
tags: [rust, mcp, stdio]

# Dependency graph
requires:
  - phase: 02-tool-surface-stdio-delivery/02-01
    provides: rmcp tool router + schemas
provides:
  - stdio MCP server binary (`docracy-mcp`)
  - stdout-safety regression test
  - OpenCode local command configuration

key-files:
  created:
    - crates/mcp/src/bin/docracy-mcp.rs
    - crates/mcp/tests/stdio_binary_smoke.rs
    - .planning/phases/02-tool-surface-stdio-delivery/02-OPENCODE.md
  modified:
    - crates/mcp/Cargo.toml
    - Cargo.lock

requirements-completed: [TRN-01, CFG-03]

# Metrics
completed: 2026-04-06
---

# Phase 02 Plan 02: stdio server binary + stdout-safety test + OpenCode config Summary

Shipped a stdio MCP server binary (`docracy-mcp`) that boots Docracy via the shared `docracy-mcp` runtime and serves rmcp tools over stdio, with strict stdout discipline.

## Task Commits

1. **Task 1: stdio server binary**
   - `d918f6f` feat(02-02): add stdio MCP server binary

2. **Task 2: stdout-safety regression test**
   - `d3a5153` test(02-02): enforce stdout-safety on startup errors

3. **Task 3: OpenCode local config**
   - `e717a25` docs(02-02): add OpenCode stdio config

## Verification

- `cargo build -p docracy-mcp --bin docracy-mcp` ✅
- `cargo test -p docracy-mcp --test stdio_binary_smoke` ✅

## Self-Check: PASSED

- FOUND: .planning/phases/02-tool-surface-stdio-delivery/02-02-SUMMARY.md
- FOUND commits: d918f6f, d3a5153, e717a25
