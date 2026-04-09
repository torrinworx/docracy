---
phase: 11-refresh-readme-usage-docs
plan: 01
subsystem: docs
tags: [readme, usage, mcp, workspace, init]
requires:
  - phase: 10-task-scoped-init-contexts
    provides: task-scoped init output and `task_context_documents` contract
provides:
  - root README current-state and usage docs aligned to the shipped v1.1 contract
  - workspace bootstrap guidance via `docracy workspace create` and `WORKSPACE_ID`
  - copy-pasteable AGENTS.md bootstrap reminder
affects: [phase 11, README.md, agent startup docs]
tech-stack:
  added: []
  patterns: [documentation contract sync, additive init fields, CLI-only workspace bootstrap]
key-files:
  created: []
  modified: [README.md]
key-decisions:
  - "State the current milestone as v1.1 MCP Server Interface instead of v1.0"
  - "Document task-scoped init as additive fields layered onto the full active context set"
  - "Keep workspace provisioning CLI-only and hand off the UUID through WORKSPACE_ID"
patterns-established:
  - "Current-state docs should describe the shipped contract, not future ideas"
  - "Agent setup snippets must remain copy-pasteable and exact"
requirements-completed: [DOC-01]
duration: 12min
completed: 2026-04-09
---

# Phase 11: Refresh README usage docs Summary

**Root README usage docs synced to the shipped v1.1 MCP Server Interface, including task-scoped init and workspace bootstrap.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-09T12:35:00Z
- **Completed:** 2026-04-09T12:47:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Replaced stale v1.0 lead-in with the current v1.1 MCP Server Interface wording.
- Documented `task_scope` / `task_context_documents` as additive init fields.
- Added CLI-only workspace bootstrap guidance and preserved the AGENTS.md reminder block.

## Task Commits

1. **Task 1: Rewrite current-state and contract sections** - `b6f7b2c` (docs)

**Plan metadata:** `b6f7b2c` (docs)

## Files Created/Modified
- `README.md` - refreshed current-state, tool, workspace, and bootstrap guidance

## Decisions Made
- Followed the shipped contract exactly and avoided presenting future ideas as current behavior.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- The shell lacked `python`; verification was rerun with `python3` and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Root README is current for operators and agents.
- Phase 11 plan 02 can proceed with the same contract assumptions.

---
*Phase: 11-refresh-readme-usage-docs*
*Completed: 2026-04-09*

## Self-Check: PASSED
