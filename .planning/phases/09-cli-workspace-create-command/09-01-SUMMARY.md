---
phase: 09-cli-workspace-create-command
plan: 01
subsystem: cli
tags: [cli, postgres, uuid, workspace, validation, testing, docs]

# Dependency graph
requires:
  - phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation
    provides: workspace tenancy, shared/global fallback, and explicit WORKSPACE_ID binding
provides:
  - CLI workspace provisioning command for manual workspace bootstrap
  - Postgres workspace insert helper
  - Regression coverage for workspace ID parsing, stderr validation, and row creation
  - Operator docs for exporting WORKSPACE_ID into MCP startup
affects: [cli, postgres, docs, testing, mcp-bootstrap]

# Tech tracking
tech-stack:
  added: [uuid, thiserror]
  patterns: [nested CLI subcommands, pre-DB CLI validation, adapter-backed workspace inserts, structured stderr fixtures]

key-files:
  created: [crates/cli/tests/workspace_create.rs, crates/cli/tests/fixtures/create_workspace_invalid_id.stderr.json]
  modified: [crates/cli/Cargo.toml, crates/cli/src/main.rs, crates/cli/tests/cli_stderr.rs, crates/postgres/src/lib.rs, crates/postgres/tests/postgres_integration.rs, README.md, Cargo.lock, .planning/phases/09-cli-workspace-create-command/09-01-PLAN.md, .planning/phases/09-cli-workspace-create-command/09-CONTEXT.md]

key-decisions:
  - "Keep workspace provisioning CLI-only and return the created UUID as JSON on stdout"
  - "Reject malformed and nil workspace IDs before connecting to Postgres so invalid input fails fast with structured validation errors"
  - "Persist workspaces through PgRepository instead of embedding raw SQL in the CLI"

patterns-established:
  - "Pattern 1: CLI validation helpers can emit structured validation_error JSON without requiring a live database connection"
  - "Pattern 2: Postgres integration coverage can reuse the isolated schema harness to verify workspace bootstrap behavior"

requirements-completed: [WS-05, WS-06, WS-07]

# Metrics
duration: 59 min
completed: 2026-04-08
---

# Phase 09: CLI Workspace Create Command Summary

**CLI workspace provisioning with UUID generation, structured validation, and Postgres-backed row creation**

## Performance

- **Duration:** 59 min
- **Started:** 2026-04-08T21:40:00Z
- **Completed:** 2026-04-08T22:39:21Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Added `docracy workspace create` with optional `--workspace-id` and default UUID generation.
- Wired the CLI through `PgRepository::create_workspace` so workspace provisioning stays in the Postgres adapter.
- Added CLI stderr fixtures, Postgres integration coverage, and operator docs for the CLI-to-`WORKSPACE_ID` bootstrap flow.

## Task Commits

1. **Task 1: Add the CLI workspace-create command and Postgres insert helper** - `f6ed454` (feat)
2. **Task 2: Add CLI regression tests and operator docs** - `f8cc4fc` (feat)

**Plan metadata:** `45e2fde` (docs)
**Planning context:** `febb0d8` (docs)

## Files Created/Modified
- `crates/cli/Cargo.toml` - Added UUID and validation dependencies for the workspace command.
- `crates/cli/src/main.rs` - Added nested workspace subcommand parsing, workspace ID validation, and JSON output.
- `crates/postgres/src/lib.rs` - Added `create_workspace` plus nil-UUID guarding.
- `crates/postgres/tests/postgres_integration.rs` - Added workspace row creation coverage.
- `crates/cli/tests/workspace_create.rs` - Added command-level coverage for default IDs, explicit IDs, and nil UUID rejection.
- `crates/cli/tests/cli_stderr.rs` - Added malformed workspace ID stderr regression.
- `crates/cli/tests/fixtures/create_workspace_invalid_id.stderr.json` - Golden stderr fixture for invalid workspace IDs.
- `README.md` - Documented manual workspace bootstrap and `WORKSPACE_ID` handoff.
- `Cargo.lock` - Locked the new CLI dependencies.

## Decisions Made
- Kept workspace management outside MCP and scoped it to the CLI/Postgres layer only.
- Chose JSON stdout that returns the created `workspace_id` directly for easy shell export.
- Validated malformed and nil IDs before the database connection to keep invalid-input failures fast and structured.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- No local `DATABASE_URL` was configured, so database-backed tests remain env-gated; the suite still passes via the validation and fixture coverage that does not require a live database.

## User Setup Required

None - no external service configuration was added by this phase.

## Next Phase Readiness
- Operators can now provision a workspace UUID from the CLI and pass it into `WORKSPACE_ID` for MCP startup.
- Workspace provisioning remains CLI-only, so the MCP tool surface stays unchanged.

---
*Phase: 09-cli-workspace-create-command*
*Completed: 2026-04-08*

## Self-Check: PASSED

- Summary file exists.
- Task commit `f6ed454` exists.
- Task commit `f8cc4fc` exists.
