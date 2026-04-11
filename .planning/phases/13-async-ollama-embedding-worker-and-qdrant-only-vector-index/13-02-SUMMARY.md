---
phase: 13-async-ollama-embedding-worker-and-qdrant-only-vector-index
plan: 02
subsystem: database
tags: [postgres, ollama, qdrant, async, embeddings]

# Dependency graph
requires:
  - phase: 13-01
    provides: retryable embedding-job queue and workspace-scoped vector mirror contract
provides:
  - workspace-scoped async embedding worker binary
  - Ollama embed-to-Qdrant processing path with retryable failure handling
  - local worker runbook and env contract
affects: [phase 13, local operator workflow, derived vector indexing]

# Tech tracking
tech-stack:
  added: [tokio, dotenvy, reqwest]
  patterns: [workspace-scoped queue leasing, Ollama /api/embed requests, retryable Qdrant upserts]

key-files:
  created: [crates/postgres/src/indexer.rs, crates/postgres/src/bin/docracy-indexer.rs]
  modified: [crates/postgres/Cargo.toml, crates/postgres/src/lib.rs, crates/postgres/src/vector.rs, crates/postgres/tests/indexer_integration.rs, README.md, .env.example, Cargo.lock]

key-decisions:
  - "Bind the worker to a single WORKSPACE_ID and connect Postgres with session-scoped workspace state before polling."
  - "Lease jobs with FOR UPDATE SKIP LOCKED, then only delete queue rows after the Qdrant write succeeds."
  - "Store embed_model in Qdrant payloads so the derived index can be rebuilt and traced back to the model used."

patterns-established:
  - "Pattern 1: worker config comes from env defaults that match local Ollama/Qdrant compose services."
  - "Pattern 2: per-job failures are converted into retry metadata instead of dropping queue rows."

requirements-completed: [IDX-02, IDX-03]

# Metrics
duration: 8 min
completed: 2026-04-10
---

# Phase 13: Async Ollama Embedding Worker and Qdrant-Only Vector Index Summary

**Workspace-scoped embedding worker that drains canonical Postgres job rows, calls Ollama, and writes rebuildable Qdrant vectors with retry metadata.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-10T23:42:30Z
- **Completed:** 2026-04-10T23:50:17Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Added the `docracy-indexer` worker binary and env-driven runtime.
- Implemented Ollama embed requests plus workspace-scoped Qdrant collection upserts.
- Added retry semantics that preserve failed jobs with attempt/error metadata.
- Documented the local worker runbook and env contract in README and `.env.example`.

## Task Commits

1. **Task 1: Add the worker binary and queue-drain runtime** - `f662c78` (test)
2. **Task 1: Add the worker binary and queue-drain runtime** - `018e7fc` (feat)
3. **Task 2: Embed through Ollama and upsert Qdrant points** - `01bf018` (test)
4. **Task 2: Embed through Ollama and upsert Qdrant points** - `c1b5280` (feat)
5. **Task 3: Document the local worker runbook and env contract** - `237d608` (docs)

## Files Created/Modified

- `crates/postgres/src/indexer.rs` - worker config, claim loop, Ollama/Qdrant pipeline, retry handling.
- `crates/postgres/src/bin/docracy-indexer.rs` - async worker entrypoint.
- `crates/postgres/src/vector.rs` - Qdrant payload now records the embedding model.
- `crates/postgres/tests/indexer_integration.rs` - worker regression coverage.
- `README.md` - local worker runbook.
- `.env.example` - worker env contract.

## Decisions Made

- Workspace scope stays explicit through `WORKSPACE_ID` instead of guessing tenant context.
- Derived vectors remain rebuildable by keeping model and tombstone metadata in Qdrant payloads.
- Failure handling keeps jobs retryable rather than treating embed or upsert errors as terminal.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Expected red/green TDD compile failures were resolved by adding the missing worker APIs and retry plumbing.

## User Setup Required

None - local Ollama/Qdrant/Postgres settings are documented, but no new external service configuration is required beyond existing workspace setup.

## Next Phase Readiness

- The worker can be started locally with `cargo run -p docracy-postgres --bin docracy-indexer`.
- Phase 13 can now build on a canonical Postgres queue plus rebuildable Qdrant indexing.

## Self-Check: PASSED

- Summary file exists on disk.
- Task commits found in git history: `f662c78`, `018e7fc`, `01bf018`, `c1b5280`, `237d608`.

---
*Phase: 13-async-ollama-embedding-worker-and-qdrant-only-vector-index*
*Completed: 2026-04-10*
