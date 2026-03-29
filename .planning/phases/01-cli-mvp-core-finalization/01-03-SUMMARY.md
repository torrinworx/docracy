---
phase: 01-cli-mvp-core-finalization
plan: 03
subsystem: cli
tags: [json, clap, serde, docs, query]

# Dependency graph
requires:
  - phase: 01-cli-mvp-core-finalization/01-01
    provides: revision-safe update semantics and OCC checks for CLI updates
provides:
  - final CLI update payload with expected_revision input
  - structured JSON CLI errors with non-zero exits
  - README examples aligned to the shipped CLI/query contract
affects: [README, CLI contract, future query/search plans]

# Tech tracking
tech-stack:
  added: []
  patterns: ["serde alias for backward-compatible JSON fields", "structured JSON error envelope on stderr"]

key-files:
  created: []
  modified: [crates/cli/src/main.rs, README.md]

key-decisions:
  - "Expose expected_revision in CLI update JSON while preserving expected_head as a backward-compatible alias."
  - "Return structured JSON error objects from the CLI instead of plain strings."
  - "Keep README query examples explicit about v1 extension-search deferral."

patterns-established:
  - "Pattern 1: CLI input contracts can evolve with serde aliases without breaking older payloads."
  - "Pattern 2: CLI failures should serialize as machine-readable JSON envelopes."

requirements-completed: [QRY-01, QRY-02, QRY-03, QRY-04, CLI-01, CLI-02, CLI-03, PG-03]

# Metrics
duration: 13 min
completed: 2026-04-05
---

# Phase 01: CLI MVP + Core Finalization Summary

**CLI update payloads now use expected_revision, errors emit machine-readable JSON, and README examples match the v1 query contract without implying extension search support.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-04-05T20:16:30Z
- **Completed:** 2026-04-05T20:29:41Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Finalized the CLI update input contract around `expected_revision` with backward-compatible aliasing.
- Added structured JSON error output with typed `kind` values and details for revision conflicts.
- Reworked README examples so query docs stay explicit about deferred extension search in v1.

## Task Commits

1. **Task 1: Wire the final JSON CLI update contract** - `83e83b4` (feat)
2. **Task 2: Align docs with the final CLI surface** - `feebb55` (docs)

## Files Created/Modified
- `crates/cli/src/main.rs` - Accepts `expected_revision`, emits structured JSON errors, preserves database URL precedence.
- `README.md` - Shows the shipped update payload and removes v1 extension-search implications.

## Decisions Made
- Kept `expected_head` as a serde alias so old payloads still work while the public JSON contract moves to `expected_revision`.
- Standardized CLI failures on a JSON envelope to keep automation simple.
- Documented query/search semantics conservatively to avoid promising extension filtering before governance defines it.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 CLI contract is stable and machine-readable.
- Ready for Phase 2 core test harness work.

## Self-Check: PASSED

- Summary file exists on disk.
- Task commits `83e83b4` and `feebb55` exist in git history.

---
*Phase: 01-cli-mvp-core-finalization*
*Completed: 2026-04-05*
