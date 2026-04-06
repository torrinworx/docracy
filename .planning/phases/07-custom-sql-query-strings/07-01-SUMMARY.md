---
phase: 07-custom-sql-query-strings
plan: 01
subsystem: api
tags: [sql, query-routing, rust, postgres, testing]

# Dependency graph
requires:
  - phase: 04-audit-verification-closure
    provides: verified core query/service behavior and regression coverage
provides:
  - raw SQL query execution contract in the core
  - service-layer dispatch between guided and raw query modes
  - unit coverage for raw/guided parsing and routing
affects: [07-custom-sql-query-strings-02, mcp query tooling, postgres repository adapters]

# Tech tracking
tech-stack:
  added: [async_trait(?Send), RawQueryInput, RawQueryResult, QueryExecution]
  patterns: [mode-split query dispatch, raw SQL fallback hook, service-owned execution branching]

key-files:
  created: []
  modified:
    - crates/core/src/query.rs
    - crates/core/src/repository.rs
    - crates/core/src/service.rs
    - crates/core/src/lib.rs
    - crates/core/src/memory.rs
    - crates/postgres/src/lib.rs
    - crates/mcp/src/tools.rs
    - crates/mcp/tests/operations.rs
    - crates/postgres/tests/postgres_integration.rs

key-decisions:
  - "Use a typed QueryExecution enum so raw SQL and guided parsing stay explicit at the core boundary."
  - "Default repository raw-query support returns an unsupported-storage error unless an adapter overrides it."
  - "Relax async-trait futures to ?Send for repository object safety across the core and adapters."

patterns-established:
  - "QueryInput now carries optional sql and timeout_ms fields for direct execution mode."
  - "Service::query_documents owns the raw vs guided dispatch and preserves the public QueryResult shape."

requirements-completed: [TOOL-02, TOOL-03]

# Metrics
duration: 10 min
completed: 2026-04-08
---

# Phase 07: Custom SQL Query Strings Summary

**Raw SQL query escape hatch with guided fallback and service-layer routing**

## Performance

- **Duration:** 10 min
- **Started:** 2026-04-08T14:55:42Z
- **Completed:** 2026-04-08T15:05:42Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments
- Added a typed raw-query mode to `QueryInput` without breaking legacy guided parsing.
- Routed `query_documents` through either guided projection or raw repository execution.
- Added unit coverage for raw SQL precedence, guided fallback, and raw repository dispatch.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define the raw-query core contract** - `40162e2` (feat)
2. **Task 2: Route core query_documents through raw or guided execution** - `eebd8f1` (feat)
3. **Task 3: Lock the mode split with focused unit tests** - `e13ddfd` (test)

**Plan metadata:** pending

## Files Created/Modified
- `crates/core/src/query.rs` - raw/guided query contract and parse tests
- `crates/core/src/repository.rs` - default raw-query hook
- `crates/core/src/service.rs` - execution-mode dispatch and service tests
- `crates/core/src/lib.rs` - new query-contract re-exports
- `crates/core/src/memory.rs` - async-trait object-safety alignment
- `crates/postgres/src/lib.rs` - async-trait object-safety alignment
- `crates/mcp/src/tools.rs` - MCP query input now supplies sql/timeouts
- `crates/mcp/tests/operations.rs` - MCP query test updated for new contract
- `crates/postgres/tests/postgres_integration.rs` - integration query tests updated for new contract

## Decisions Made
- Used an explicit `QueryExecution` enum instead of inferring mode from JSON shape checks.
- Kept raw SQL handling in the service layer so adapters only need to implement the execution hook.
- Made the repository raw-query hook opt-in with a safe default error.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Relaxed repository async-trait futures for raw query dispatch**
- **Found during:** Task 1 (Define the raw-query core contract)
- **Issue:** Adding a default async raw-query hook caused trait-object `Sync` constraints to block `&dyn Repository` dispatch.
- **Fix:** Switched repository traits and implementations to `#[async_trait(?Send)]`.
- **Files modified:** `crates/core/src/repository.rs`, `crates/core/src/memory.rs`, `crates/postgres/src/lib.rs`
- **Verification:** `cargo test -p docracy-core --lib`
- **Committed in:** `40162e2` (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for trait-object compatibility; no scope creep.

## Issues Encountered
- None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- The core now exposes a raw SQL execution mode and still preserves the guided query fallback.
- Phase 07-02 can build on the new execution split without reworking query parsing or repository dispatch.

## Self-Check: PASSED

- Summary file exists on disk.
- All three task commits exist in git history.
- The plan changes were verified with `cargo test -p docracy-core --lib`.

---
*Phase: 07-custom-sql-query-strings*
*Completed: 2026-04-08*
