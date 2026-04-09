---
phase: 11-refresh-readme-usage-docs
plan: 02
subsystem: docs
tags: [mcp, readme, contract, startup, errors]
requires:
  - phase: 10-task-scoped-init-contexts
    provides: task-scoped init output and `task_context_documents` contract
provides:
  - MCP README contract and startup docs aligned with the shipped runtime
  - tool payload examples that match the current CLI/core shapes
  - error semantics aligned with structured `ErrorData` payloads
affects: [phase 11, crates/mcp/README.md, MCP client authors]
tech-stack:
  added: []
  patterns: [documentation contract sync, startup env contract, structured error docs]
key-files:
  created: []
  modified: [crates/mcp/README.md]
key-decisions:
  - "Describe WORKSPACE_ID as the process-lifetime binding for MCP sessions"
  - "Document DOCRACY_TASK_SCOPE as optional and additive rather than a replacement for context_documents"
  - "Keep update payload docs centered on expected_revision and alias expected_head"
patterns-established:
  - "MCP docs should state transport/runtime behavior without implying unsupported surfaces"
  - "Tool examples must mirror shipped field names and precedence rules exactly"
requirements-completed: [DOC-01]
duration: 11min
completed: 2026-04-09
---

# Phase 11: Refresh README usage docs Summary

**MCP README startup, tool, and error contract docs now match the shipped workspace-binding and task-scoped init behavior.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-04-09T12:47:00Z
- **Completed:** 2026-04-09T12:58:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Clarified `WORKSPACE_ID` binding, shared/global fallback, and additive `DOCRACY_TASK_SCOPE` startup behavior.
- Aligned the MCP tool examples with `expected_revision`, `expected_head`, and raw SQL precedence.
- Documented `ErrorData` and machine-readable `kind` values for failures.

## Task Commits

1. **Task 1: Update workspace binding and task-scope docs** - `e079a57` (docs)

**Plan metadata:** `e079a57` (docs)

## Files Created/Modified
- `crates/mcp/README.md` - refreshed startup, tool, and error contract docs

## Decisions Made
- Kept the shipped tool surface unchanged and documented only the current contract.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- The shell lacked `python`; verification was rerun with `python3` and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- MCP-facing docs are aligned with the runtime and core payload shapes.
- Phase 11 is ready for final state updates and metadata commit.

---
*Phase: 11-refresh-readme-usage-docs*
*Completed: 2026-04-09*

## Self-Check: PASSED
