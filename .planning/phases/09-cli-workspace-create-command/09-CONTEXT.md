---
phase: 09-cli-workspace-create-command
gathered: 2026-04-08
status: ready-for-planning
source: user-request-followup
---

# Phase 09: CLI Workspace Create Command - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning
**Source:** Follow-up to Phase 08 workspace tenancy

<domain>
## Phase Boundary

Add a CLI-only workspace provisioning command so operators can create a workspace row and UUID manually before binding that UUID into `WORKSPACE_ID` for MCP sessions.

</domain>

<decisions>
## Implementation Decisions

### CLI surface
- Add `docracy workspace create` as a nested CLI command.
- Accept `--workspace-id <uuid>` as an optional override for scripted provisioning.
- Generate a UUID by default when no ID is provided.

### Storage behavior
- Persist workspaces through the existing Postgres adapter.
- Reuse the reserved global zero UUID only for the shared fallback row; reject it from user-created workspaces.

### Error and output shape
- Keep stdout JSON and stderr structured JSON errors aligned with the current CLI contract.
- Invalid UUID input should fail fast before insert.

### Scope control
- Do not add any MCP workspace-management tools.
- Keep this phase out of `docracy_core`; workspace creation is an operational CLI/Postgres concern.

</decisions>

<canonical_refs>
## Canonical References

### Roadmap / requirements
- `.planning/ROADMAP.md` — phase scope and sequencing for the CLI workspace creation follow-up.
- `.planning/REQUIREMENTS.md` — workspace lifecycle requirements to be added for the CLI-only provisioning command.

### Workspace tenancy foundation
- `migrations/0006_workspace_tenancy.sql` — reserved global workspace row and tenant-scoped schema.
- `crates/postgres/src/lib.rs` — workspace-bound Postgres adapter pattern to reuse for inserts.
- `crates/mcp/README.md` — existing workspace-binding contract that must remain MCP-only for session scope.

### CLI shape
- `crates/cli/src/main.rs` — current CLI command layout and JSON error envelope.
- `README.md` — user-facing command documentation that should explain the manual provisioning flow.

</canonical_refs>

<specifics>
## Specific Ideas

- `workspace create` should be the primary manual bootstrap command for operators.
- Workspace UUIDs should be generated with a UUID library by default, not invented by path heuristics.
- The command should return the created workspace ID so it can be copied into `WORKSPACE_ID`.

</specifics>

<deferred>
## Deferred Ideas

- Workspace list / delete / rename commands.
- Any MCP tool exposure for workspace provisioning.
- Admin UI or HTTP workspace lifecycle endpoints.

</deferred>

---

*Phase: 09-cli-workspace-create-command*
*Context gathered: 2026-04-08 via follow-up planning*
