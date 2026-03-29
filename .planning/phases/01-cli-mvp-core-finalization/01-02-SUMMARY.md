---
phase: 01-cli-mvp-core-finalization
plan: 02
subsystem: governance
tags: [rust, governance, constitution, validation]

# Dependency graph
requires:
  - phase: 01-01
    provides: revision OCC and document/revision update plumbing
provides:
  - rerunnable init that seeds and repairs the repo-owned constitution
  - active context documents returned from init
  - public update guard for constitution documents
affects: [01-03, phase-2 core tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - system-only reconciliation path for repo-owned governance documents
    - shared validation helper for reserved mutable document types

key-files:
  created: []
  modified:
    - crates/core/src/service.rs
    - crates/core/src/document.rs
    - crates/core/src/validation.rs

key-decisions:
  - "Init repairs constitution state through an internal helper instead of the public update path."
  - "Reserved constitution validation is centralized for mutable user input while keeping the stored constitution system-managed."

patterns-established:
  - "System-only document reconciliation may bypass user-facing mutation rules when bootstrapping repo-owned governance data."
  - "Shared validation helpers keep reserved document-type rules explicit and stable."

requirements-completed: [DOC-02, DOC-04, DOC-05, GOV-01, GOV-02, GOV-03, GOV-04]

# Metrics
duration: 15 min
completed: 2026-04-05
---

# Phase 01: CLI MVP + Core Finalization Summary

Rerunnable init now seeds and repairs the repo-owned constitution without exposing public mutation paths, while user-facing validation keeps constitution documents reserved.

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-05T20:20:00Z
- **Completed:** 2026-04-05T20:35:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Init now reconciles constitution state through an internal system-only path.
- Public document updates reject constitution mutations with the existing reserved-type error.
- Active context documents are returned from init alongside governance files.

## Task Commits

1. **Task 1: Make init rerunnable and constitution-safe** - `d498545` (fix)
2. **Task 2: Reject constitution changes through user-facing document input** - `88f7f37` (refactor)

**Plan metadata:** pending

## Files Created/Modified
- `crates/core/src/service.rs` - System-only constitution reconciliation and public update guard.
- `crates/core/src/document.rs` - Shared mutable document-type validation helper.
- `crates/core/src/validation.rs` - Reserved-type validation helper and coverage.

## Decisions Made
- Kept constitution repair inside init so normal document mutation rules stay strict.
- Centralized mutable document-type checks into a shared validation helper.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Init is rerunnable and deterministic for governance seeding.
- Constitution remains repo-owned and immutable through user-facing paths.
- Ready for 01-03.

## Self-Check: PASSED
