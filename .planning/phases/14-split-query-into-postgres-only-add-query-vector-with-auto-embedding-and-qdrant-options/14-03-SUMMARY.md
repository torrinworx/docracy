---
phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
plan: 03
subsystem: cli
tags: [rust, cli, query, vector_search, ollama, qdrant]

# Dependency graph
requires:
  - phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
    provides: QueryVectorInput + query_vector_documents + ollama_embed_text
provides:
  - CLI `query-vector` subcommand with embedding-or-query auto-embedding contract
  - CLI `query` remains Postgres-only (guided + raw SQL)
affects: [mcp, docs, query, vector]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - CLI-level query_vector JSON contract decoupled from core contracts

key-files:
  created: []
  modified:
    - crates/cli/src/main.rs
    - crates/core/src/lib.rs

key-decisions:
  - "Expose query_vector_documents at the docracy_core crate root so interface crates can call it consistently"

patterns-established:
  - "query_vector requires either embedding or query text; auto-embedding uses docracy_postgres::ollama_embed_text"

requirements-completed: [TBD]

# Metrics
duration: 3m
completed: 2026-04-11
---

# Phase 14 Plan 03: CLI query-vector entrypoint Summary

**CLI now exposes `query-vector` for semantic search with optional auto-embedding, while keeping `query` strictly Postgres-only.**

## Performance

- **Duration:** 3m
- **Started:** 2026-04-11T01:07:27Z
- **Completed:** 2026-04-11T01:09:51Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Added `docracy query-vector` subcommand that accepts either an explicit embedding or `query` text (auto-embedded via Ollama).
- Wired the CLI to call core `query_vector_documents` and return the same `QueryResult` JSON shape as `query`.
- Ensured `docracy query` remains Postgres-only by continuing to parse `QueryInput` without any embedding routing.

## Task Commits

1. **Task 1: Add CLI QueryVector subcommand + auto-embedding input contract** - `17786e5` (feat)

## Files Created/Modified

- `crates/cli/src/main.rs` - Adds `QueryVector` subcommand, input parsing, auto-embedding, and core delegation.
- `crates/core/src/lib.rs` - Re-exports `query_vector_documents` for interface-layer calls.

## Decisions Made

- Re-export core use-cases (`query_vector_documents`) at the crate root to keep CLI/MCP call-sites consistent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Re-exported `query_vector_documents` from `docracy_core`**
- **Found during:** Task 1
- **Issue:** CLI plan referenced `docracy_core::query_vector_documents`, but the function was only available via `docracy_core::service`.
- **Fix:** Added a crate-root re-export in `crates/core/src/lib.rs`.
- **Verification:** `cargo test -p docracy-cli` and workspace `cargo build` succeed.

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Minimal; enabled the planned CLI wiring without changing behavior.

## Issues Encountered

- Workspace `cargo build` initially failed due to pending MCP updates (resolved in Plan 14-04).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- MCP surface can mirror the CLI split (`query` vs `query_vector`).

## Self-Check: PASSED

- FOUND: .planning/phases/14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options/14-03-SUMMARY.md
- FOUND commit: 17786e5
