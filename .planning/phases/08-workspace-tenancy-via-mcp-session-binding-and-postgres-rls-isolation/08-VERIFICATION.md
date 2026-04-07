---
phase: 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation
verified: 2026-04-08T20:39:28Z
status: passed
score: 6/6 must-haves verified
---

# Phase 08: Workspace Tenancy via MCP Session Binding and Postgres RLS Isolation Verification Report

**Phase Goal:** Define workspace tenancy for Docracy with generated workspace IDs, explicit MCP session binding through project-scoped client config and `WORKSPACE_ID`, and Postgres RLS isolation so each session only sees its active workspace while shared governance stays in the global scope.

**Verified:** 2026-04-08T20:39:28Z
**Status:** passed
**Re-verification:** No

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Global governance rows remain visible while workspace rows stay isolated across sessions with different bindings. | ✓ VERIFIED | `workspace_scoped_sessions_isolate_reads_queries_and_raw_sql` passes; migration reserves the zero UUID global workspace and applies RLS. |
| 2 | Every document and revision row carries workspace scope and Postgres RLS blocks cross-workspace reads and writes. | ✓ VERIFIED | `workspace_id` columns/defaults, composite FKs, workspace-leading indexes, and forced RLS are present in `migrations/0006_workspace_tenancy.sql`. |
| 3 | Raw SQL queries obey the same workspace boundary as normal repository calls. | ✓ VERIFIED | Integration test proves raw SQL only returns scoped rows; `query_raw_documents` runs through the same pooled connection. |
| 4 | MCP startup can read an explicit workspace UUID from `WORKSPACE_ID` and keep it in process-wide runtime state. | ✓ VERIFIED | `crates/mcp/src/bin/docracy-mcp.rs`, `crates/mcp/src/config.rs`, and `crates/mcp/src/runtime.rs` thread `workspace_id` through bootstrap; config tests pass. |
| 5 | Project-scoped OpenCode config can pass `WORKSPACE_ID` to the MCP server without filesystem-derived identity heuristics. | ✓ VERIFIED | `opencode.json` passes `WORKSPACE_ID` via `{env:DOCRACY_WORKSPACE_ID}`. |
| 6 | The docs explain the workspace-binding contract and preserve the shared/global fallback model. | ✓ VERIFIED | `crates/mcp/README.md` documents shared/global vs workspace-bound startup, fallback behavior, and fixed `./governance`. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `migrations/0006_workspace_tenancy.sql` | workspace tenancy schema + RLS | ✓ VERIFIED | Implements reserved global workspace, `workspace_id` columns, `current_setting('docracy.workspace_id', true)` fallback, and RLS. Implemented as `0006` because `0004` was already occupied. |
| `crates/postgres/src/lib.rs` | workspace-scoped Postgres connection helper | ✓ VERIFIED | `connect_scoped()` installs `docracy.workspace_id` through `after_connect`. |
| `crates/postgres/tests/postgres_integration.rs` | workspace isolation regression coverage | ✓ VERIFIED | Covers scoped isolation, raw SQL, and shared governance visibility; targeted test passed. |
| `crates/mcp/src/config.rs` | workspace-aware startup config parsing | ✓ VERIFIED | Carries `workspace_id` and parses optional UUIDs. |
| `crates/mcp/src/runtime.rs` | workspace-bound bootstrap wiring | ✓ VERIFIED | Boots the scoped repo and stores workspace state for process lifetime. |
| `crates/mcp/src/bin/docracy-mcp.rs` | environment-driven MCP server startup | ✓ VERIFIED | Reads `WORKSPACE_ID`, preserves stderr-only setup errors, and passes binding into startup config. |
| `opencode.json` | project-scoped client config example | ✓ VERIFIED | Explicit `WORKSPACE_ID` env passthrough is checked in. |
| `crates/mcp/README.md` | workspace-binding documentation | ✓ VERIFIED | Explains binding contract, fallback, and shared governance visibility. |
| `crates/mcp/tests/stdio_binary_smoke.rs` | startup safety regression | ✓ VERIFIED | Exercises workspace env path and confirms stdout stays empty on startup failure. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `PgRepository::connect_scoped` | `SET LOCAL/ set_config('docracy.workspace_id', ...)` | session-scoped custom GUC | ✓ WIRED | `after_connect` installs the workspace ID on pooled connections. |
| `documents.workspace_id` / `document_revisions.workspace_id` | RLS policy | `current_setting('docracy.workspace_id', true)` | ✓ WIRED | Policies allow only the active workspace plus the global workspace. |
| raw SQL execution | workspace filters | same RLS policy | ✓ WIRED | Scoped raw SQL hits the same connection/session, so RLS applies uniformly. |
| `McpStartupConfig.workspace_id` | `PgRepository::connect_scoped` | bootstrap passes bound tenant into adapter | ✓ WIRED | `bootstrap()` forwards the parsed workspace UUID into the repository constructor. |
| `opencode.json` | `WORKSPACE_ID` | project-scoped MCP environment block | ✓ WIRED | Config uses env substitution from `DOCRACY_WORKSPACE_ID`. |
| stdio startup | stderr-only setup errors | transport safety envelope | ✓ WIRED | Setup failures are serialized to stderr; stdout remains reserved for MCP messages. |

### Requirements Coverage

Note: `WS-01`..`WS-04` are phase-local IDs from the roadmap/plans and do not exist in `.planning/REQUIREMENTS.md`. They are accounted for here as orphaned traceability entries.

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| WS-01 | 08-01 | Workspace schema, RLS policies, and scope-aware Postgres harness | ORPHANED / accounted for | Implemented in `migrations/0006_workspace_tenancy.sql` and verified by `postgres_integration` tests. |
| WS-02 | 08-02 | MCP workspace binding, project config env wiring, and operator docs | ORPHANED / accounted for | Implemented in `crates/mcp/src/config.rs`, `runtime.rs`, `docracy-mcp.rs`, `opencode.json`, and `README.md`. |
| WS-03 | 08-01 | Workspace isolation across repository reads, queries, and raw SQL | ORPHANED / accounted for | Verified by `workspace_scoped_sessions_isolate_reads_queries_and_raw_sql`. |
| WS-04 | 08-02 | Startup safety and shared/global fallback under workspace binding | ORPHANED / accounted for | Verified by config tests, stdio smoke test, and docs. |

### Anti-Patterns Found

None found in the modified phase files.

### Human Verification Required

None.

### Gaps Summary

No blocking gaps found. The phase goal is achieved: database tenancy is enforced by RLS, MCP startup binds workspaces explicitly, project-scoped client config passes `WORKSPACE_ID`, and shared/global governance remains available.

---

_Verified: 2026-04-08T20:39:28Z_
_Verifier: the agent (gsd-verifier)_
