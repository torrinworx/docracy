---
phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation
plan: 02
subsystem: infra
tags: [mcp, postgres, uuid, opencode, rls]

# Dependency graph
requires:
  - phase: 08-01
    provides: workspace-scoped Postgres adapter and tenant GUC binding
provides:
  - Workspace-aware MCP startup config and bootstrap wiring
  - Project-scoped OpenCode example with explicit `WORKSPACE_ID` passthrough
  - MCP README workspace-binding contract
  - Stdio smoke test covering workspace-bound startup failures
affects: [mcp, opencode, postgres, workspace-tenancy]

# Tech tracking
tech-stack:
  added: [uuid]
  patterns: [startup workspace binding, env-driven tenant selection, shared/global fallback]

key-files:
  created: [".planning/phases/08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation/08-02-SUMMARY.md"]
  modified: ["crates/mcp/src/config.rs", "crates/mcp/src/runtime.rs", "crates/mcp/src/bin/docracy-mcp.rs", "opencode.json", "crates/mcp/README.md", "crates/mcp/tests/stdio_binary_smoke.rs", "Cargo.lock", "crates/mcp/Cargo.toml"]

key-decisions:
  - "Bind tenant scope from WORKSPACE_ID at process startup and keep missing values on the shared/global fallback path."
  - "Store the bound workspace on McpRuntime so the process lifetime retains the session scope alongside the scoped repository."
  - "Use project-scoped OpenCode env substitution instead of repository-path heuristics for client selection."

patterns-established:
  - "Pattern 1: MCP startup validates env-driven workspace UUIDs before launching the transport."
  - "Pattern 2: Workspace-bound and shared/global sessions share one bootstrap path and one setup-error envelope."

requirements-completed: [WS-02, WS-04]

# Metrics
duration: 35 min
completed: 2026-04-08
---

# Phase 08: Workspace Tenancy via MCP Session Binding and Postgres RLS Isolation Summary

**Workspace-bound MCP startup with project-scoped `WORKSPACE_ID` and shared/global fallback**

## Performance

- **Duration:** 35 min
- **Started:** 2026-04-08T20:32:00Z
- **Completed:** 2026-04-08T21:07:00Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments
- MCP startup now parses `WORKSPACE_ID`, carries it in startup/runtime state, and connects through the scoped Postgres adapter.
- The checked-in OpenCode config now passes workspace identity explicitly via env substitution.
- The MCP README and stdio smoke test both preserve the shared/global fallback and transport-safe setup behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add workspace binding to MCP startup config and bootstrap** - `6722716` (feat)
2. **Task 2: Document the workspace-binding contract in project config and MCP README** - `b0b37c2` (docs)
3. **Task 3: Lock the stdio startup safety under a workspace-bound environment** - `f56067d` (test)

## Files Created/Modified
- `crates/mcp/src/config.rs` - Added workspace UUID startup config and parsing helper.
- `crates/mcp/src/runtime.rs` - Bootstraps the scoped repository and stores workspace state.
- `crates/mcp/src/bin/docracy-mcp.rs` - Reads `WORKSPACE_ID` and keeps setup errors on stderr.
- `opencode.json` - Project-scoped MCP env example now passes `WORKSPACE_ID`.
- `crates/mcp/README.md` - Documents shared/global vs workspace-bound startup contract.
- `crates/mcp/tests/stdio_binary_smoke.rs` - Exercises the workspace env path.
- `crates/mcp/Cargo.toml` / `Cargo.lock` - Added `uuid` dependency.

## Decisions Made
- Used `WORKSPACE_ID` as the explicit startup binding source, not path heuristics.
- Kept missing `WORKSPACE_ID` as a valid shared/global mode.
- Reused the scoped Postgres adapter from 08-01 instead of adding another tenant-selection layer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing `uuid` dependency for MCP workspace parsing**
- **Found during:** Task 1 (workspace binding implementation)
- **Issue:** `crates/mcp/src/config.rs` referenced `uuid::Uuid` before the crate depended on `uuid`.
- **Fix:** Added `uuid` to `crates/mcp/Cargo.toml` and refreshed `Cargo.lock`.
- **Files modified:** `crates/mcp/Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo test -p docracy-mcp --lib config` passed.
- **Committed in:** `6722716` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary plumbing only; no scope creep.

## Issues Encountered
- `requirements mark-complete` could not map the plan-local WS-02/WS-04 IDs into `.planning/REQUIREMENTS.md`; requirement traceability remains phase-local for now.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- MCP startup, docs, and smoke coverage now agree on the workspace-binding contract.
- Remaining work in the milestone can build on an explicit tenant-scope startup path.

## Self-Check: PASSED

---
*Phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation*
*Completed: 2026-04-08*
