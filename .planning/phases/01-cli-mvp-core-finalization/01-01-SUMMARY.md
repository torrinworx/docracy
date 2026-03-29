---
phase: 01-cli-mvp-core-finalization
plan: 01
subsystem: database
tags: [rust, postgres, sqlx, cli, revisions, occ]

# Dependency graph
requires: []
provides:
  - Expected-head revision checks for updates
  - Atomic revision chaining for memory and Postgres adapters
  - CLI update input contract now carries the head revision
affects:
  - Phase 01-02 governance init
  - Phase 01-03 CLI/query finalization
  - Phase 02 core harness

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optimistic concurrency control via expected-head revision checks"
    - "Transactional revision chaining with rollback on stale writes"

key-files:
  created:
    - .planning/phases/01-cli-mvp-core-finalization/01-01-SUMMARY.md
  modified:
    - crates/core/src/service.rs
    - crates/core/src/repository.rs
    - crates/core/src/memory.rs
    - crates/postgres/src/lib.rs
    - crates/core/src/errors.rs
    - crates/cli/src/main.rs
    - crates/postgres/tests/postgres_integration.rs

key-decisions:
  - "Use RevisionConflict in core so stale writes fail before revision composition and carry explicit expected/actual head data."
  - "Check the persisted head inside the Postgres transaction before writing revisions."
  - "Keep the in-memory adapter aligned with repository-level OCC checks for local usage and tests."

requirements-completed: [DOC-01, DOC-03, REV-01, REV-02, REV-03, REV-04, PG-01, PG-02]

# Metrics
duration: 4 min
completed: 2026-04-05
---

# Phase 01 Plan 01: CLI MVP + Core Finalization Summary

**Revision-safe document updates now require an expected head revision across the core service, CLI input, and Postgres adapter, with stale writes rejected before new revisions are chained.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-05T20:07:50Z
- **Completed:** 2026-04-05T20:11:51Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Update flow now requires a head revision and rejects stale writes explicitly.
- Memory and Postgres repositories enforce the same OCC rule.
- Revision chaining still advances atomically from v1 to v2 on success.

## Task Commits

1. **Task 1: Require an expected head revision for updates** - `f4dc4e4`
2. **Task 2: Enforce OCC in the adapters** - `a2fb91b`

## Files Created/Modified

- `crates/core/src/service.rs` - update contract and stale-head check
- `crates/core/src/repository.rs` - repository OCC signature
- `crates/core/src/errors.rs` - explicit revision conflict error
- `crates/core/src/memory.rs` - in-memory stale-write rejection
- `crates/postgres/src/lib.rs` - transactional stale-write rejection
- `crates/cli/src/main.rs` - CLI update input now carries expected head
- `crates/postgres/tests/postgres_integration.rs` - update test input contract
- `.planning/phases/01-cli-mvp-core-finalization/01-01-SUMMARY.md` - plan summary

## Decisions Made

- Used an explicit `RevisionConflict` core error to report stale heads with expected/actual context.
- Checked the persisted head inside the Postgres transaction before writing revisions.
- Kept the in-memory adapter aligned with repository-level OCC checks for local usage and tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Propagated expected_head through CLI/integration inputs**
- **Found during:** Task 1
- **Issue:** The widened update contract broke CLI and integration compilation until callers supplied the required head revision.
- **Fix:** Added `expected_head` to the CLI update input and Postgres integration test input.
- **Files modified:** `crates/cli/src/main.rs`, `crates/postgres/tests/postgres_integration.rs`
- **Verification:** `cargo test -p docracy-cli --no-run`, `cargo test -p docracy-postgres --no-run`

**2. [Rule 1 - Bug] Fixed in-memory OCC check against persisted head**
- **Found during:** Task 2
- **Issue:** The in-memory repository initially compared the updated document state instead of the stored head, causing a false conflict.
- **Fix:** Check the persisted document head before applying the replacement.
- **Files modified:** `crates/core/src/memory.rs`
- **Verification:** `cargo test -p docracy-core`
- **Committed in:** `a2fb91b`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Necessary to keep the new update contract compiling and to make OCC behavior correct in local and Postgres-backed execution.

## Issues Encountered

- `cargo fmt --check` required a formatting pass; `cargo fmt` resolved it.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Core update semantics are now OCC-safe.
- Phase 01-02 can build on the stabilized revision contract.
- No known blockers remain for the next plan.

## Self-Check: PASSED

- Summary file exists on disk.
- Task commits `f4dc4e4` and `a2fb91b` are present in git history.
