---
phase: 10-task-scoped-init-contexts
plan: 01
subsystem: core
tags: [init, contexts, governance, task-scopes]

# Dependency graph
requires:
  - phase: 09-cli-workspace-create-command
    provides: workspace-aware core state and current init contract baseline
provides:
  - core Init result with task-scoped context subset fields
  - init_bundle_scoped helper for interface layers
affects: [mcp, init, planning, context-selection]

# Tech tracking
tech-stack:
  added: []
  patterns: ["scope-aware init helper", "extension-based task context filtering"]

key-files:
  created: []
  modified: [crates/core/src/service.rs, crates/core/src/lib.rs]

key-decisions:
  - "Preserve the existing init contract by keeping context_documents as the full active context set and adding task_context_documents as an opt-in subset."
  - "Use extensions.task_scopes as the only task selector so specialty init contexts stay data-driven and do not require a new tool surface."

patterns-established:
  - "Pattern 1: add a scoped helper next to the existing core entrypoint, then re-export it for interfaces."
  - "Pattern 2: treat unscoped context docs as universally eligible while excluding non-matching scoped docs."

requirements-completed: ["TBD"]

# Metrics
duration: 18 min
completed: 2026-04-09
---

# Phase 10: task-scoped-init-contexts Summary

Core init now returns a task-scoped subset of active context documents without changing the full active-context contract.

## Performance

- **Duration:** 18 min
- **Started:** 2026-04-09T04:03:58Z
- **Completed:** 2026-04-09T04:21:58Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Added `init_bundle_scoped(..., task_scope)` alongside the existing `init_bundle` path.
- Extended `InitBundleResult` with `task_scope` and `task_context_documents`.
- Added unit coverage for no-scope and planning-scope filtering behavior.

## Task Commits

1. **Task 1: Extend InitBundleResult with task-scoped context subset** - `495f156` (feat)

**Plan metadata:** pending

## Files Created/Modified
- `crates/core/src/service.rs` - adds scoped init helper, task-scoped result fields, and unit tests.
- `crates/core/src/lib.rs` - re-exports the scoped init helper.

## Decisions Made
- Keep init contract compatibility by returning the full active context set unchanged.
- Use `extensions.task_scopes` as an opt-in filter for task-specific context selection.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Scoped init helper is available for the next interface-layer plan.
- The next plan can build on deterministic task-scope filtering without changing active-context semantics.

## Self-Check: PASSED

---
*Phase: 10-task-scoped-init-contexts*
*Completed: 2026-04-09*
