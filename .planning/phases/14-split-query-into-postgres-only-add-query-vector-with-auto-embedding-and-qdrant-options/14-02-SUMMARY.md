---
phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
plan: 02
subsystem: postgres-adapter
tags: [rust, postgres, qdrant, ollama, embeddings, vector_search]

# Dependency graph
requires:
  - phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
    provides: Core QueryVectorInput + query_vector_documents split
provides:
  - ollama_embed_text helper for generating embeddings from query text
  - Bounded Qdrant candidate multiplier to reduce post-filter dropouts
  - Postgres integration tests migrated to QueryVectorInput + query_vector_documents
affects: [cli, mcp, vector, query]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Env-configured embed helper with local TcpListener request-capture tests
    - Vector candidate over-fetching with clamp bounds

key-files:
  created: []
  modified:
    - crates/postgres/src/vector.rs
    - crates/postgres/src/lib.rs
    - crates/postgres/tests/postgres_integration.rs

key-decisions:
  - "Keep Ollama embed request/response parsing in the Postgres crate so CLI/MCP can reuse it without duplicating JSON handling"
  - "Over-fetch Qdrant candidates (bounded) and truncate after Postgres filtering to reduce empty/short result sets"

patterns-established:
  - "Vector search filtering clears DocumentQuery.query before Postgres filtering to avoid accidental FTS constraints"

requirements-completed: [TBD]

# Metrics
duration: 6m
completed: 2026-04-11
---

# Phase 14 Plan 02: Postgres adapter updates for split vector query Summary

**Postgres adapter now provides an Ollama embedding helper and improves Qdrant vector search robustness under the new core vector query entrypoint.**

## Performance

- **Duration:** 6m
- **Started:** 2026-04-11T01:01:58Z
- **Completed:** 2026-04-11T01:07:27Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `ollama_embed_text` helper (env-configured) to turn query text into an embedding via `POST /api/embed`.
- Hardened `PgRepository::query_vector_documents` by (a) clearing the text query before Postgres filtering and (b) requesting more Qdrant candidates than the final limit (bounded) to reduce post-filter dropouts.
- Migrated Postgres vector integration tests to `QueryVectorInput` + core `query_vector_documents`.

## Task Commits

1. **Task 1: Add and export an Ollama embed helper for query text**
   - `152b7e2` (test)
   - `39fafdf` (feat)
2. **Task 2: Update PgRepository vector query + migrate tests to QueryVectorInput**
   - `6199482` (test)
   - `a1f5580` (feat)

## Files Created/Modified

- `crates/postgres/src/vector.rs` - Adds `ollama_embed_text` and a request-capture unit test.
- `crates/postgres/src/lib.rs` - Re-exports embed helper; improves `query_vector_documents` candidate behavior.
- `crates/postgres/tests/postgres_integration.rs` - Uses `QueryVectorInput` + `query_vector_documents` for vector search coverage.

## Decisions Made

- Keep embed helper in the adapter crate so interface layers can call it consistently with the indexer’s parsing rules.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo test -p docracy-postgres ollama_embed_text` initially failed because integration tests still referenced the removed `QueryInput.embedding`; resolved as part of Task 2 test migration.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Interface layers can implement a `query_vector` operation by calling `ollama_embed_text` then core `query_vector_documents`.

## Self-Check: PASSED

- FOUND: .planning/phases/14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options/14-02-SUMMARY.md
- FOUND commits: 152b7e2, 39fafdf, 6199482, a1f5580
