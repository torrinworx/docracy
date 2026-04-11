---
phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
plan: 04
subsystem: mcp
tags: [rust, mcp, query, vector_search, ollama, qdrant]

# Dependency graph
requires:
  - phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
    provides: Core/adapter split (QueryInput vs QueryVectorInput) + embed helper
provides:
  - MCP `query` tool is Postgres-only (no embedding field)
  - MCP `query_vector` tool with embedding-or-query auto-embedding contract
  - Updated MCP docs and tool contract tests
affects: [opencode, openwebui, cli, query, vector]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Tool-surface split: `query` (Postgres) vs `query_vector` (vector)

key-files:
  created: []
  modified:
    - crates/mcp/src/tools.rs
    - crates/mcp/src/operations.rs
    - crates/mcp/tests/operations.rs
    - crates/mcp/tests/tool_contract.rs
    - crates/mcp/README.md

key-decisions:
  - "Keep auto-embedding inside the MCP tool handler using docracy_postgres::ollama_embed_text to match CLI behavior"

patterns-established:
  - "QueryVectorArgs validates embedding/query presence and maps failures to McpErrorKind::ValidationError"

requirements-completed: [TBD]

# Metrics
duration: 2m
completed: 2026-04-11
---

# Phase 14 Plan 04: MCP query_vector tool Summary

**MCP now mirrors the core split: `query` is Postgres-only, and `query_vector` performs vector search with optional Ollama auto-embedding.**

## Performance

- **Duration:** 2m
- **Started:** 2026-04-11T01:10:47Z
- **Completed:** 2026-04-11T01:12:16Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Removed `embedding` from MCP `query` args and mapped it directly to the Postgres-only core `QueryInput`.
- Added MCP `query_vector` tool with `QueryVectorArgs` (embedding or query text + embed_model), delegating to:
  - `docracy_postgres::ollama_embed_text` for auto-embedding
  - `docracy_core::query_vector_documents` for vector querying
- Updated MCP README and tool contract tests to include the new tool.

## Task Commits

1. **Task 1: Remove embedding from MCP query + add query_vector tool**
   - `151ef1c` (test)
   - `2828d64` (feat)

## Files Created/Modified

- `crates/mcp/src/tools.rs` - Removes embedding from `QueryArgs`; adds `QueryVectorArgs` + `query_vector` handler.
- `crates/mcp/src/operations.rs` - Adds delegation helper for `query_vector_documents`.
- `crates/mcp/tests/operations.rs` - Updates query delegation test; adds validation coverage for query_vector args.
- `crates/mcp/tests/tool_contract.rs` - Updates stable tools list expectation.
- `crates/mcp/README.md` - Documents `query_vector` and clarifies `query` is Postgres-only.

## Decisions Made

- Implement auto-embedding in the MCP tool handler using the Postgres embed helper for consistent JSON parsing and env defaults.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated tool contract stability test to include query_vector**
- **Found during:** Task 1
- **Issue:** Adding a new tool broke `tools_list_is_stable` expectations.
- **Fix:** Updated `crates/mcp/tests/tool_contract.rs` tool list to include `query_vector`.
- **Verification:** `cargo test -p docracy-mcp --quiet`.

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Minimal; updated regression test to match the new intended tool surface.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CLI and MCP now both expose explicit vector query entrypoints; next work can focus on higher-level client integration and/or schema docs.

## Self-Check: PASSED

- FOUND: .planning/phases/14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options/14-04-SUMMARY.md
- FOUND commits: 151ef1c, 2828d64
