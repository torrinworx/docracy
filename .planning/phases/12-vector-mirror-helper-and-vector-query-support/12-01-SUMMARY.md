---
phase: 12-vector-mirror-helper-and-vector-query-support
plan: 01
subsystem: database
tags: [postgres, vector, qdrant, rust, sqlx]

# Dependency graph
requires:
  - phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation
    provides: workspace-scoped Postgres storage and session binding
provides:
  - storage-agnostic vector mirror contracts
  - current-state vector mirror queue table
  - Postgres queueing of mirror snapshots during document writes
affects:
  - phase 12-02
  - future Qdrant dispatch/query work

# Tech tracking
tech-stack:
  added: []
  patterns:
    - current-state workspace/document upsert queue for derived embeddings
    - embedding payload opt-in via document extensions
    - archive/delete state mirrored in queue rows

key-files:
  created:
    - crates/core/src/vector.rs
    - migrations/0007_vector_mirror_queue.sql
  modified:
    - crates/core/src/lib.rs
    - crates/postgres/src/lib.rs
    - crates/postgres/tests/postgres_integration.rs

key-decisions:
  - "Represent vector mirroring as a current snapshot keyed by workspace + document instead of an append-only log."
  - "Use document extensions.embedding as the opt-in payload source so vector mirroring stays storage-agnostic."
  - "Mirror archive/delete state in the queue row and overwrite rows in place with ON CONFLICT."

requirements-completed: [VEC-01]

# Metrics
duration: 6 min
completed: 2026-04-10
---

# Phase 12: Vector Mirror Helper and Vector Query Support Summary

**Workspace-scoped vector mirror snapshots now queue in Postgres as current-state rows with archive-aware overwrites and a storage-agnostic core contract.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-10T03:03:50Z
- **Completed:** 2026-04-10T03:10:13Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added a core `VectorMirrorRecord` / `VectorQueryInput` contract for future vector mirroring and search.
- Added a Postgres `vector_mirror_queue` table keyed by workspace and document, with in-transaction upserts.
- Added integration coverage for snapshot overwrite, workspace isolation, and archive/unarchive transitions.

## Task Commits

1. **Task 1: Define the vector mirror snapshot contract** - `a144d94` (feat)
2. **Task 2: Queue current mirror snapshots from Postgres writes** - `0e13a98` (feat)
3. **Task 3: Prove archive and workspace scoping on the queued snapshot** - `f25f170` (feat)

## Files Created/Modified
- `crates/core/src/vector.rs` - vector mirror/query contract types
- `crates/core/src/lib.rs` - re-exports for vector contracts
- `migrations/0007_vector_mirror_queue.sql` - current-state vector mirror queue schema
- `crates/postgres/src/lib.rs` - queue extraction + transactional upsert helpers
- `crates/postgres/tests/postgres_integration.rs` - queue overwrite/isolation coverage

## Decisions Made
- Use `extensions.embedding` as the opt-in payload carrier so the core document model stays unchanged.
- Keep mirror rows current-only with `(workspace_id, document_id)` as the unique key.
- Store embedding payloads as JSONB arrays plus an explicit dimension column for later Qdrant dispatch.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered
- The local environment did not provide `DOCRACY_TEST_DATABASE_URL`, so Postgres integration tests compiled successfully but skipped at runtime.
- A small pure helper was added in the Postgres adapter so the embedding-to-snapshot contract could still be unit-tested without a live database.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 12-02 can reuse the core vector contract and queue schema.
- Live Postgres verification is still required in an environment with `DOCRACY_TEST_DATABASE_URL`.

## Self-Check: PASSED

---
*Phase: 12-vector-mirror-helper-and-vector-query-support*
*Completed: 2026-04-10*
