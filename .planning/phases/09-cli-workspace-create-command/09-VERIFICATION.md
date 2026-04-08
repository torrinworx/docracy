---
phase: 09-cli-workspace-create-command
verified: 2026-04-08T22:42:57Z
status: passed
score: 3/3 must-haves verified
---

# Phase 09: CLI Workspace Create Command Verification Report

**Phase Goal:** Add a CLI-only workspace management command that provisions a workspace row and returns its UUID so operators can bind `WORKSPACE_ID` manually, while leaving the MCP tool surface unchanged.
**Verified:** 2026-04-08T22:42:57Z
**Status:** passed
**Re-verification:** No

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The CLI can create a workspace row and return its UUID without involving MCP. | ✓ VERIFIED | `crates/cli/src/main.rs` adds `docracy workspace create` and returns `{"workspace_id": ...}`; `crates/postgres/src/lib.rs` provides `PgRepository::create_workspace`; `README.md` documents the manual bootstrap flow. |
| 2 | Workspace IDs are generated with a UUID library by default, and reserved nil UUIDs stay tied to the shared global fallback only. | ✓ VERIFIED | `crates/cli/src/main.rs` uses `Uuid::new_v4()` when no ID is supplied and rejects nil UUIDs; `crates/cli/Cargo.toml` adds `uuid`; `crates/postgres/src/lib.rs` also guards nil IDs. |
| 3 | The workspace-create flow keeps the existing structured stdout/stderr contract and surfaces invalid IDs as CLI validation errors. | ✓ VERIFIED | `crates/cli/tests/cli_stderr.rs` and `crates/cli/tests/fixtures/create_workspace_invalid_id.stderr.json` lock the stderr envelope; `crates/cli/tests/workspace_create.rs` covers success and nil-ID rejection. |

**Score:** 3/3 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/cli/src/main.rs` | Nested `workspace create` command + JSON output wiring | ✓ VERIFIED | CLI parses `workspace create`, resolves/validates workspace IDs, calls `repo.create_workspace`, and prints `workspace_id` JSON on stdout. |
| `crates/cli/Cargo.toml` | UUID dependency for CLI workspace generation/parsing | ✓ VERIFIED | `uuid = { version = "1.8", features = ["v4"] }` is present. |
| `crates/postgres/src/lib.rs` | Workspace insert helper on the Postgres adapter | ✓ VERIFIED | `create_workspace()` validates nil UUIDs and executes `INSERT INTO workspaces (id) VALUES ($1)`. |
| `crates/postgres/tests/postgres_integration.rs` | Workspace row insertion coverage | ✓ VERIFIED | `create_workspace_inserts_workspace_row()` creates a workspace and asserts it exists in Postgres. |
| `crates/cli/tests/workspace_create.rs` | Workspace-create CLI regression coverage | ✓ VERIFIED | Covers generated IDs, explicit UUIDs, and nil UUID rejection. |
| `crates/cli/tests/cli_stderr.rs` | Structured stderr validation for bad workspace IDs | ✓ VERIFIED | Compares malformed workspace UUID stderr against a golden JSON fixture. |
| `crates/cli/tests/fixtures/create_workspace_invalid_id.stderr.json` | Golden stderr fixture | ✓ VERIFIED | Contains the expected `validation_error` JSON for invalid workspace IDs. |
| `README.md` | Manual operator flow for generating and exporting workspace IDs | ✓ VERIFIED | Documents `docracy workspace create`, `--workspace-id`, and exporting the result into `WORKSPACE_ID`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `docracy workspace create` | `INSERT INTO workspaces (id)` | Postgres-only workspace provisioning helper | WIRED | CLI command calls `repo.create_workspace()`, and the adapter performs the insert. |
| `workspace_id argument` | `Uuid::new_v4()` default generation | CLI generates an ID when one is omitted | WIRED | `resolve_workspace_id(None)` returns `Uuid::new_v4()`. |
| `invalid workspace id` | structured stderr JSON | existing CLI error envelope | WIRED | Malformed IDs become `validation_error` JSON on stderr; fixture-backed regression locks output. |
| `workspace provisioning docs` | `WORKSPACE_ID` | manual bootstrap path for later MCP binding | WIRED | README tells operators to export the returned UUID into `WORKSPACE_ID` before starting MCP. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| WS-05 | 09-01-PLAN.md | Operators can create a workspace row through the CLI and receive a UUID for later `WORKSPACE_ID` binding. | ✓ SATISFIED | CLI command returns `workspace_id` JSON and Postgres helper inserts the row. |
| WS-06 | 09-01-PLAN.md | Workspace creation uses a generated UUID by default, accepts an explicit UUID for scripted provisioning, and keeps the reserved nil UUID mapped to the shared global workspace only. | ✓ SATISFIED | `resolve_workspace_id()` generates/validates IDs; nil UUID is rejected in CLI and adapter. |
| WS-07 | 09-01-PLAN.md | Workspace provisioning stays CLI-only; the MCP tool surface remains `Init/Create/Read/Query/Update` and does not gain workspace management tools. | ✓ SATISFIED | README states MCP remains unchanged; no MCP workspace-management command was added. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `README.md` | 326 | `TODO/Later` future-ideas note | Info | Non-blocking roadmap note; does not affect the workspace-create flow. |

### Human Verification Required

None.

### Gaps Summary

No blocking gaps found. The CLI provisions workspaces directly, UUID generation/validation is in place, the Postgres adapter owns insertion, regression tests cover success and invalid-input paths, and operator docs show how to hand the UUID to `WORKSPACE_ID` for MCP startup.

---

_Verified: 2026-04-08T22:42:57Z_
_Verifier: the agent (gsd-verifier)_
