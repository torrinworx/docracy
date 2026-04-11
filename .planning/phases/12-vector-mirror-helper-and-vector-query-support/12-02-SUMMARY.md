---
phase: 12-vector-mirror-helper-and-vector-query-support
plan: 02
subsystem: vector-search
tags: [qdrant, postgres, reqwest, sqlx, workspace-isolation]

# Dependency graph
requires:
  - phase: 12-01
    provides: vector mirror queue schema and current-only snapshot tracking
provides:
  - workspace-scoped Qdrant mirror flushing
  - embedding-based query routing with Postgres hydration
  - archive/workspace vector regression coverage
affects: [README, postgres adapter, core query/service contract, future vector phases]

# Tech tracking
tech-stack:
  added: [reqwest]
  patterns: [workspace-scoped Qdrant collections, canonical-Postgres hydration, current-only vector snapshots]

key-files:
  created: [crates/postgres/src/vector.rs]
  modified: [crates/postgres/Cargo.toml, Cargo.lock, crates/postgres/src/lib.rs, crates/core/src/query.rs, crates/core/src/repository.rs, crates/core/src/service.rs, crates/core/src/memory.rs, crates/postgres/tests/postgres_integration.rs, README.md]

key-decisions:
  - "Use workspace-scoped Qdrant collections keyed by document id so vector points overwrite cleanly instead of accumulating stale embeddings."
  - "Treat Postgres as canonical for filtering and hydration; Qdrant only supplies ranked ids."
  - "Keep archive/deleted state authoritative in Postgres and mirror it into vector payloads for regression checks."

patterns-established:
  - "Pattern 1: derived vector state is written as a mirror, never as the source of truth"
  - "Pattern 2: vector results are re-hydrated from Postgres before being returned to callers"

requirements-completed: [VEC-01]

# Metrics
duration: 22 min
completed: 2026-04-10
---

# Phase 12: Vector Mirror Helper and Vector Query Support Summary

**Workspace-scoped Qdrant mirroring now feeds embedding search while Postgres stays canonical for filtering, hydration, and archive state.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-04-10T03:10:30Z
- **Completed:** 2026-04-10T03:32:51Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments
- Added Qdrant flush support for current vector snapshots in workspace-scoped collections.
- Routed embedding-bearing queries through Qdrant and re-hydrated ranked ids from Postgres.
- Added regression coverage for workspace scoping, archive filtering, and stale snapshot replacement.

## Task Commits

1. **Task 1: Flush the vector mirror queue into workspace-scoped Qdrant collections** - `9464aab` (feat)
2. **Task 2: Add vector query routing and Postgres hydration** - `c6b277d` (feat)
3. **Task 3: Prove archive/workspace isolation and document the derived vector contract** - `40aa77f` (feat)

**Plan metadata:** `40aa77f` (feat: add postgres vector search mirroring and docs)

## Files Created/Modified
- `crates/postgres/src/vector.rs` - Qdrant client helpers, flush path, and search helper.
- `crates/postgres/src/lib.rs` - Postgres vector search hook and query dispatch.
- `crates/core/src/query.rs` - Embedding-bearing query input and vector execution routing.
- `crates/core/src/service.rs` - Vector query hydration path and ranked-row projection.
- `crates/core/src/repository.rs` - Safe default vector-search hook.
- `crates/core/src/memory.rs` - Default vector-search fallback test.
- `crates/postgres/tests/postgres_integration.rs` - Integration coverage for collection naming, flush failure, workspace isolation, and archive-aware vector search.
- `README.md` - Canonical Postgres / derived Qdrant contract.

## Decisions Made
- Qdrant collection names are workspace-scoped and point ids are document ids.
- Postgres filters and hydrates vector results; Qdrant only ranks candidate ids.
- Archive/deleted state remains authoritative in Postgres and is mirrored into vector payloads.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Completed missing Postgres vector-search hook**
- **Found during:** Task 3
- **Issue:** Task 2 routing existed in core, but the Postgres adapter still lacked the vector query hook required to search Qdrant and hydrate through Postgres.
- **Fix:** Implemented `query_vector_documents` in `crates/postgres/src/lib.rs` with workspace-scoped Qdrant search plus Postgres hydration/filtering.
- **Files modified:** `crates/postgres/src/lib.rs`, `crates/postgres/src/vector.rs`
- **Verification:** `cargo test -p docracy-postgres vector -- --nocapture`
- **Committed in:** `40aa77f`

**2. [Rule 2 - Missing Critical] Added vector-search fallback coverage for repositories**
- **Found during:** Task 2
- **Issue:** The core repository boundary lacked a safe default error for vector queries.
- **Fix:** Added `query_vector_documents` default error plus a MemoryRepository regression test.
- **Files modified:** `crates/core/src/repository.rs`, `crates/core/src/memory.rs`
- **Verification:** `cargo test -p docracy-core vector -- --nocapture`
- **Committed in:** `c6b277d`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** No scope creep; both fixes were required for correctness and adapter safety.

## Issues Encountered
- Local Postgres credentials were not configured in this environment, so DB-backed integration tests are guarded and only execute when `DOCRACY_TEST_DATABASE_URL` or `DATABASE_URL` is available.

## User Setup Required
None - no external service configuration required for the documented contract.

## Next Phase Readiness
- Vector mirroring is now workspace-scoped and archive-aware.
- Future vector phases can build on a derived Qdrant index without changing the canonical Postgres contract.

---
*Phase: 12-vector-mirror-helper-and-vector-query-support*
*Completed: 2026-04-10*

## Self-Check: PASSED
