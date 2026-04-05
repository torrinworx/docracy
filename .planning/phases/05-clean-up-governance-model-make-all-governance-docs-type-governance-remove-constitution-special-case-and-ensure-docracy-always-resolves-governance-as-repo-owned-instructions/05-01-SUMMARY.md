---
phase: 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions
plan: 01
subsystem: database
tags: [governance, postgres, migrations, rust]

# Dependency graph
requires:
  - phase: 04-audit-verification-closure
    provides: audit baseline with shipped v1 verification evidence
provides:
  - reserved governance document type in core
  - data migration from `constitution` rows to `governance`
  - postgres regression coverage for governance bootstrap and uniqueness
affects: [05-02, governance startup, postgres init/migration]

# Tech tracking
tech-stack:
  added: [migrations/0005_governance_type_cleanup.sql]
  patterns: [reserved governance doc type, fixed governance uniqueness index]

key-files:
  created: [migrations/0005_governance_type_cleanup.sql]
  modified: [crates/core/src/document.rs, crates/core/src/validation.rs, crates/core/src/errors.rs, crates/core/src/memory.rs, crates/core/src/service.rs, crates/cli/src/main.rs, crates/mcp/src/error.rs, crates/postgres/tests/postgres_integration.rs]

key-decisions:
  - "Rename the reserved repo-owned instructions model to governance while leaving the on-disk bundle filename untouched for now."
  - "Use a forward migration that rewrites existing constitution rows and enforces one governance row via a dedicated unique index."

patterns-established:
  - "Pattern 1: Governance is the reserved document type everywhere in core validation and repository enforcement."
  - "Pattern 2: Postgres migration tests assert the repaired active document, duplicate conflict, and index shape."

requirements-completed: [GOV-05, GOV-06]

# Metrics
duration: 5min
completed: 2026-04-06
---

# Phase 5: Governance Cleanup Summary

Governance now speaks in `governance` terms end-to-end in core and persisted rows migrate forward without breaking init/update flows.

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-06T15:40:07-04:00
- **Completed:** 2026-04-06T15:44:30-04:00
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Reserved document handling now uses `DocumentType::GOVERNANCE` and `ReservedGovernanceType`.
- Existing constitution rows are rewritten to governance and the single-governance unique index is enforced.
- Postgres integration covers governance bootstrap, duplicate insert conflict, and normal document flows.

## Task Commits

1. **Task 1: Rename the reserved governance doc type in core** - `2a8462f` (fix)
2. **Task 2: Migrate stored constitution rows to governance** - `98af8d0` + `4c78964` (fix)

## Files Created/Modified
- `crates/core/src/document.rs` - reserved governance type constant and validation test rename
- `crates/core/src/validation.rs` - governance-specific reserved type validation
- `crates/core/src/errors.rs` - governance bundle error rename
- `crates/core/src/memory.rs` - single governance-row enforcement
- `crates/core/src/service.rs` - governance init/repair path and tests
- `crates/cli/src/main.rs` - governance error translation alignment
- `crates/mcp/src/error.rs` - MCP governance error translation alignment
- `migrations/0005_governance_type_cleanup.sql` - row rewrite + unique index migration
- `crates/postgres/tests/postgres_integration.rs` - governance migration and duplicate insert regression coverage

## Decisions Made
- Kept the repo-owned instructions bundle filename in place and changed the reserved document type/model instead.
- Chose a forward migration that both rewrites data and recreates the reserved-type uniqueness guard.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Core rename required downstream error translation updates**
- **Found during:** Task 1 (Rename the reserved governance doc type in core)
- **Issue:** Renaming the core governance error variant broke CLI/MCP translation paths.
- **Fix:** Updated CLI and MCP error mapping to `MissingGovernance` / `MissingGovernance` and adjusted user-facing messages.
- **Files modified:** `crates/cli/src/main.rs`, `crates/mcp/src/error.rs`
- **Committed in:** `2a8462f`

**2. [Rule 3 - Blocking] Existing migration slot 0004 was already occupied**
- **Found during:** Task 2 (Migrate stored constitution rows to governance)
- **Issue:** The repository already had `0004_repository_invariants.sql`, so the planned migration filename conflicted.
- **Fix:** Added the cleanup migration as `0005_governance_type_cleanup.sql` instead, preserving forward-only ordering.
- **Files modified:** `migrations/0005_governance_type_cleanup.sql`
- **Committed in:** `98af8d0`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Necessary compatibility fixes; no scope creep.

## Issues Encountered
- None beyond the blocking rename and migration numbering conflict handled above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Core governance naming and stored-data migration are in place.
- Phase 5 can now move to fixed governance startup path wiring and docs.

---
*Phase: 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions*
*Completed: 2026-04-06*

## Self-Check: PASSED
