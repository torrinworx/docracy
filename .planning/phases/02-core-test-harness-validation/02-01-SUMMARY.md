---
phase: 02-core-test-harness-validation
plan: 01
subsystem: testing
tags: [rust, tests, core, query, fixtures]

# Dependency graph
requires:
  - phase: 01-cli-mvp-core-finalization
    provides: core document/revision invariants, governance init, and query semantics to validate directly
provides:
  - reusable deterministic fixtures for direct-core service tests
  - direct-core query contract tests for defaults, filters, cursor round-trips, and projection
affects: [phase-2 integration coverage, future validation-driven hardening]

# Tech tracking
tech-stack:
  added: []
  patterns: ["shared test fixture module", "direct-core query-contract assertions"]

key-files:
  created: []
  modified: [crates/core/src/service.rs, crates/core/src/query.rs]

key-decisions:
  - "Consolidate deterministic service fixtures into one shared test module so create/update/init cases read as a single harness."
  - "Lock query defaults and projection behavior directly in `query.rs` instead of relying on the CLI surface."

patterns-established:
  - "Direct-core tests should reuse deterministic clock/id fixtures instead of repeating ad-hoc setup."
  - "Query semantics can be validated with parser/projection unit tests that stay CLI-independent."

requirements-completed: [TST-01, TST-03]

# Metrics
duration: 8 min
completed: 2026-04-05
---

# Phase 02 Plan 01: Core Test Harness + Validation Summary

**Reusable deterministic core fixtures now cover revision chaining, init repair, and query semantics directly inside `docracy_core`, without depending on the CLI.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-05T20:36:00Z
- **Completed:** 2026-04-05T20:44:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Consolidated the service tests around shared deterministic clock/id fixtures and a seeded-document helper.
- Added query-contract tests for default active filtering, archived/deleted handling, cursor round-trips, and extension-filter rejection.
- Kept the entire harness anchored to `docracy_core` primitives.

## Task Commits

1. **Task 1: Consolidate deterministic service fixtures** - `0902152` (test)
2. **Task 2: Lock query semantics with direct-core tests** - `0902152` (test)

**Plan metadata:** `f204e1a` (docs)

## Files Created/Modified
- `crates/core/src/service.rs` - Shared deterministic fixture module and seeded-document helper.
- `crates/core/src/query.rs` - Query parser/projection unit tests.

## Decisions Made
- Centralized deterministic test data in one fixture section to keep core service tests readable and reusable.
- Validated query behavior at the parser/projection layer so the contract remains independent of the CLI.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Core service and query behavior are locked by direct tests.
- Ready for broader Postgres-backed validation coverage.

---
*Phase: 02-core-test-harness-validation*
*Completed: 2026-04-05*

## Self-Check: PASSED

- Summary file exists on disk.
- Task commits `0902152` and `5850e73` exist in git history.
