---
phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation
plan: 01
subsystem: database
tags: [postgres, rls, tenancy, sqlx, uuid]

# Dependency graph
requires:
  - phase: 07-custom-sql-query-strings
    provides: read-only raw SQL execution with server ceilings
provides:
  - workspace-scoped Postgres schema with a reserved global workspace
  - session-bound connection helper for workspace-aware pools
  - regression coverage proving cross-workspace isolation and shared governance visibility
affects: [08-02, mcp-session-binding, postgres-adapter]

# Tech tracking
tech-stack:
  added: [uuid]
  patterns: [session-scoped custom GUC, reserved global workspace, FORCE row level security, composite workspace foreign keys, workspace-leading indexes]

key-files:
  created: [migrations/0006_workspace_tenancy.sql]
  modified: [crates/postgres/Cargo.toml, crates/postgres/src/lib.rs, crates/postgres/tests/postgres_integration.rs]

key-decisions:
  - "Use a reserved global workspace row and shared fallback for unbound sessions so governance stays readable."
  - "Bind workspace identity with PgPool after_connect set_config('docracy.workspace_id', ...) on each pooled connection."
  - "Enforce workspace-consistent document/revision lineage with composite foreign keys and RLS forced on both tables."

patterns-established:
  - "Pattern 1: workspace scope is session state, not caller-provided query filters."
  - "Pattern 2: global governance rows stay in the reserved workspace and remain visible to every session."
  - "Pattern 3: raw SQL uses the same connection/session as repository calls, so RLS applies uniformly."

requirements-completed: [WS-01, WS-03]

# Metrics
duration: 20 min
completed: 2026-04-08
---

# Phase 08: Workspace Tenancy via MCP Session Binding and Postgres RLS Isolation Summary

**Postgres workspace tenancy with session-bound GUCs and RLS isolation for shared governance.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-04-08T20:10:18Z
- **Completed:** 2026-04-08T20:30:18Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added a new workspace-tenancy migration with a reserved global workspace and RLS policies.
- Added workspace-scoped Postgres connection setup via `docracy.workspace_id` session state.
- Proved isolation across repository reads, query paths, and raw SQL while preserving shared governance access.

## Task Commits

1. **Task 1: Add the workspace tenancy migration and RLS policy** - `aa085a9` (feat)
2. **Task 2: Add a workspace-scoped Postgres connection helper and regression tests** - `5ea9067` (feat)

**Plan metadata:** pending

## Files Created/Modified

- `migrations/0006_workspace_tenancy.sql` - workspace table, scoped defaults, constraints, indexes, and RLS
- `crates/postgres/Cargo.toml` - added `uuid` dependency for workspace binding
- `crates/postgres/src/lib.rs` - workspace-scoped pool connection helper
- `crates/postgres/tests/postgres_integration.rs` - workspace isolation regression coverage

## Decisions Made

- Reserved the zero UUID as the shared global workspace so init/governance remains available to unbound sessions.
- Used `after_connect` session setup instead of caller-side filters so raw SQL inherits the same boundary.
- Added composite workspace constraints to keep document/revision lineage consistent at the database layer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Migration filename updated to match existing numbering**
- **Found during:** Task 1
- **Issue:** `migrations/0004_workspace_tenancy.sql` was requested, but `0004` already exists for repository invariants.
- **Fix:** Created `migrations/0006_workspace_tenancy.sql` so the new tenant migration runs after existing schema hardening.
- **Files modified:** `migrations/0006_workspace_tenancy.sql`
- **Verification:** `cargo test -p docracy-postgres --test postgres_integration workspace` passed.
- **Committed in:** `aa085a9`

**2. [Rule 2 - Missing Critical] Added workspace-consistent lineage constraints**
- **Found during:** Task 1
- **Issue:** Workspace-scoped rows also needed database-enforced parent/current revision consistency.
- **Fix:** Added composite workspace foreign keys and unique constraints for documents and revisions.
- **Files modified:** `migrations/0006_workspace_tenancy.sql`
- **Verification:** Postgres integration tests passed.
- **Committed in:** `aa085a9`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both fixes were required for correctness and compatibility; no scope creep beyond workspace isolation.

## Issues Encountered

- `requirements mark-complete` could not find WS-01/WS-03 in the requirements tracker, so requirement state was not updated automatically.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Workspace tenancy is enforced by the database, not by caller convention.
- Phase 08-02 can now wire MCP session binding onto the same workspace-scoped adapter.

---
*Phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation*
*Completed: 2026-04-08*

## Self-Check: PASSED
