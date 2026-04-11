---
phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
plan: 01
subsystem: core
tags: [rust, core, query, postgres, sql, vector, embeddings]

# Dependency graph
requires:
  - phase: 07-custom-sql-query-strings
    provides: Raw SQL query execution contract
  - phase: 12-vector-mirror-helper-and-vector-query-support
    provides: Repository vector query boundary + hydration approach
provides:
  - Postgres-only QueryInput (guided + raw SQL only)
  - QueryVectorInput contract requiring explicit embedding
  - query_vector_documents use-case that hydrates ranked vector hits
affects: [mcp, cli, query, vector]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Explicit split between Postgres query and vector query use-cases

key-files:
  created: []
  modified:
    - crates/core/src/query.rs
    - crates/core/src/service.rs
    - crates/core/src/lib.rs

key-decisions:
  - "Make vector search explicit by removing embedding routing from QueryInput and introducing QueryVectorInput + query_vector_documents"

patterns-established:
  - "Vector query returns ranked ids from repo and rehydrates docs via get_documents, then re-orders by ranked ids"

requirements-completed: [TBD]

# Metrics
duration: 5m
completed: 2026-04-11
---

# Phase 14 Plan 01: Split core query contracts (Postgres vs vector) Summary

**Core query is now Postgres-only (guided + raw SQL), with a separate vector query use-case that requires an explicit embedding.**

## Performance

- **Duration:** 5m
- **Started:** 2026-04-11T00:54:53Z
- **Completed:** 2026-04-11T00:59:38Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Removed hidden vector routing from `QueryInput` so `query_documents` is strictly Postgres-backed.
- Added `QueryVectorInput` (embedding-required) and `query_vector_documents` as an explicit core use-case.
- Updated/added core tests proving the split and validating ranked hydration behavior.

## Task Commits

_Note: TDD tasks include test (RED) then feature (GREEN) commits._

1. **Task 1: Make QueryInput Postgres-only (guided + raw SQL)**
   - `eccfc79` (test)
   - `04e3c05` (feat)
2. **Task 2: Add QueryVectorInput + core query_vector_documents use-case**
   - `ac2086c` (test)
   - `96a4c61` (feat)

## Files Created/Modified

- `crates/core/src/query.rs` - Removes vector routing from `QueryInput`; adds `QueryVectorInput` parsing.
- `crates/core/src/service.rs` - Keeps `query_documents` Postgres-only; adds `query_vector_documents` use-case + tests.
- `crates/core/src/lib.rs` - Re-exports `QueryVectorInput`.

## Decisions Made

- Make vector search an explicit operation: interface layers generate embeddings and call `query_vector_documents` instead of sending `embedding` through `QueryInput`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for interface layers (CLI/MCP) to add a dedicated vector-query tool/command that obtains embeddings and calls `query_vector_documents`.

## Self-Check: PASSED

- FOUND: .planning/phases/14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options/14-01-SUMMARY.md
- FOUND commits: eccfc79, 04e3c05, ac2086c, 96a4c61
