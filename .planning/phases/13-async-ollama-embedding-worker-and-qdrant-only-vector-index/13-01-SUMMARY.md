---
phase: 13-async-ollama-embedding-worker-and-qdrant-only-vector-index
plan: 01
subsystem: database
tags: [postgres, sqlx, embeddings, qdrant, ollama]

# Dependency graph
requires:
  - phase: 12-vector-mirror-helper-and-vector-query-support
    provides: current vector mirror helpers, workspace scoping, and Postgres test harnesses
provides:
  - embedding job snapshot contract for async worker handoff
  - workspace-scoped retryable embedding job queue
  - integration coverage for overwrite and tombstone snapshots
affects: [phase-13-async-ollama-embedding-worker-and-qdrant-only-vector-index, postgres adapter, future worker runtime]

# Tech tracking
tech-stack:
  added: [none]
  patterns: [canonical JSON source text snapshotting, conflict-overwrite queue upserts, workspace-scoped job leasing boundary]

key-files:
  created: [migrations/0008_embedding_job_queue.sql, crates/postgres/tests/embedding_queue_integration.rs]
  modified: [crates/core/src/vector.rs, crates/core/src/lib.rs, crates/postgres/src/lib.rs]

key-decisions:
  - "Use EmbeddingJobRecord plus canonical JSON text so the worker receives a stable snapshot of the document payload."
  - "Key embedding jobs by workspace/document/model and reset pending metadata on overwrite so stale work is replaced in place."
  - "Keep the existing vector mirror queue path alongside the new embedding queue for compatibility with the current vector-mirror phase."

patterns-established:
  - "Pattern 1: document writes enqueue a retryable embedding snapshot in the same transaction as the canonical write."
  - "Pattern 2: queue rows are overwritten in place by the latest revision instead of appending duplicate pending jobs."
  - "Pattern 3: integration tests query the queue table directly and assert archive/delete metadata propagation."

requirements-completed: [IDX-01, IDX-03]

# Metrics
duration: 25min
completed: 2026-04-10
---

# Phase 13 Plan 01: async-ollama-embedding-worker-and-qdrant-only-vector-index Summary

**Retryable embedding job snapshots now get enqueued from document writes with canonical JSON source text, ready for the async Ollama worker.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-04-10T23:17:00Z
- **Completed:** 2026-04-10T23:42:01Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added `EmbeddingJobRecord` and a canonical JSON text helper in the core crate.
- Added `embedding_job_queue` with workspace-scoped overwrite semantics.
- Proved overwrite and tombstone propagation with focused Postgres integration tests.

## Task Commits

1. **Task 1: Define the embedding job snapshot contract** - `cdf3c0a` (feat)
2. **Task 2: Add the embedding job queue and enqueue path** - `0d5b9a2` (feat)
3. **Task 3: Prove queue overwrite and retryable snapshot behavior** - `b51fbc6` (feat)

**Plan metadata:** `pending` (docs)

## Files Created/Modified
- `crates/core/src/vector.rs` - embedding job record and canonical source text helper
- `crates/core/src/lib.rs` - re-exported embedding snapshot APIs
- `migrations/0008_embedding_job_queue.sql` - new retryable job queue table
- `crates/postgres/src/lib.rs` - document write path now enqueues embedding snapshots
- `crates/postgres/tests/embedding_queue_integration.rs` - queue overwrite and tombstone regression tests

## Decisions Made
- Use a fixed default embed model (`embeddinggemma`) for queued snapshots until the worker owns runtime configuration.
- Reset attempt/error metadata when a newer revision overwrites a pending job row.
- Preserve the legacy vector mirror enqueue path for now so the current vector-mirror tests remain intact.

## Deviations from Plan

### Scope Compatibility

**1. Legacy vector mirror enqueue retained alongside the new embedding queue**
- **Found during:** Task 2
- **Issue:** The plan focused on the new embedding-job boundary, but the existing vector-mirror tests still exercise the older queue path.
- **Fix:** Kept the existing mirror enqueue path in place while adding the new embedding job queue and snapshot helper.
- **Files modified:** `crates/postgres/src/lib.rs`
- **Verification:** `cargo test -p docracy-postgres embedding_queue -- --nocapture`
- **Committed in:** `0d5b9a2`

---

**Total deviations:** 1 compatibility choice
**Impact on plan:** New queue contract is in place; legacy mirror behavior remains until the worker phase finishes the full transition.

## Issues Encountered
- None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The embedding job snapshot shape is now shared between core and Postgres.
- Phase 13 plan 02 can consume the new queue without redefining the snapshot contract.

---
*Phase: 13-async-ollama-embedding-worker-and-qdrant-only-vector-index*
*Completed: 2026-04-10*

## Self-Check: PASSED

- Summary file exists.
- Task commits exist: `cdf3c0a`, `0d5b9a2`, `b51fbc6`.
