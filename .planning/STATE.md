---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: MCP Server Interface
status: unknown
stopped_at: Completed 13-01-PLAN.md
last_updated: "2026-04-10T23:42:25.288Z"
progress:
  total_phases: 16
  completed_phases: 13
  total_plans: 27
  completed_plans: 27
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-06)

**Core value:** Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).
**Current focus:** Phase 12 — vector-mirror-helper-and-vector-query-support

## Current Position

Phase: 12 (vector-mirror-helper-and-vector-query-support) — EXECUTING
Plan: 2 of 2

## Performance Metrics

**Velocity:**

- Total plans completed: 9
- Average duration: 8 min
- Total execution time: 1.2 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| Phase 01-cli-mvp-core-finalization | 3 | 32 min | 11 min |
| Phase 02-core-test-harness-validation | 2 | 13 min | 6.5 min |
| Phase 03-stabilization-gap-closure | 1 | 10 min | 10 min |
| Phase 04-audit-verification-closure | 1 | 12 min | 12 min |

**Recent Trend:**

- Last 5 plans: Phase 02 P02, Phase 03 P01, Phase 04 P01
- Trend: v1.0 closed cleanly; planning has shifted to the MCP interface milestone

| Phase 01-cli-mvp-core-finalization P01 | 4 min | 2 tasks | 7 files |
| Phase 01-cli-mvp-core-finalization P02 | 15 min | 2 tasks | 3 files |
| Phase 01-cli-mvp-core-finalization P03 | 13 min | 2 tasks | 2 files |
| Phase 02-core-test-harness-validation P01 | 8 min | 2 tasks | 2 files |
| Phase 02-core-test-harness-validation P02 | 5 min | 2 tasks | 1 file |
| Phase 03-stabilization-gap-closure P01 | 10 min | 3 tasks | 4 files |
| Phase 04 P01 | 12 min | 3 tasks | 6 files |
| Phase 01-mcp-crate-interface-boundary P01 | 1 min | 3 tasks | 7 files |
| Phase 01-mcp-crate-interface-boundary P02 | 4 min | 3 tasks | 6 files |
| Phase 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions P01 | 5m | 2 tasks | 9 files |
| Phase 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions P02 | 5m | 2 tasks | 7 files |
| Phase 06-craft-launch-marketing-plan P01 | 8 min | 1 tasks | 1 files |
| Phase 07-custom-sql-query-strings P01 | 10 min | 3 tasks | 9 files |
| Phase 07-custom-sql-query-strings P02 | 6 min | 3 tasks | 5 files |
| Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation P01 | 20 min | 2 tasks | 4 files |
| Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation P02 | 35 min | 3 tasks | 8 files |
| Phase 09-cli-workspace-create-command P01 | 59 min | 2 tasks | 9 files |
| Phase 10-task-scoped-init-contexts P01 | 18 min | 1 tasks | 2 files |
| Phase 10-task-scoped-init-contexts P02 | 1 min | 2 tasks | 2 files |
| Phase 10-task-scoped-init-contexts P03 | 3 min | 2 tasks | 6 files |
| Phase 11 P01 | 12 min | 1 tasks | 1 files |
| Phase 11 P02 | 11 min | 1 tasks | 1 files |
| Phase 12-vector-mirror-helper-and-vector-query-support P01 | 6 min | 3 tasks | 5 files |
| Phase 12-vector-mirror-helper-and-vector-query-support P02 | 22 min | 3 tasks | 9 files |
| Phase 13 P01 | 25 min | 3 tasks | 5 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Phase 1: Commit to immutable revision history + OCC as correctness baseline
- Phase 1: CLI becomes the primary delivery surface for the core MVP
- Phase 2: Constitution is repo-owned + immutable; init must enforce DB alignment
- Phase 2: Direct-core testing harness validates implementation without CLI dependence
- Phase 2: Deterministic core tests now cover query parsing/projection and adapter-backed init/query behavior
- Phase 3: Reusable isolated-schema integration testing is ready for validation-driven hardening
- Phase 3: Deferred repository triggers enforce revision lineage without breaking transactional create/update flows
- Phase 3: CLI migration gating is command-aware, keeping `migrate` functional under `--no-migrate`
- Phase 3: README contract now matches shipped `Read` and `Update` JSON shapes
- [Phase 01-cli-mvp-core-finalization]: Use an explicit RevisionConflict core error to report stale heads with expected/actual context.
- [Phase 01-cli-mvp-core-finalization]: Check the persisted head inside the Postgres transaction before writing revisions.
- [Phase 01-cli-mvp-core-finalization]: Keep the in-memory adapter aligned with repository-level OCC checks for local usage and tests.
- [Phase 01-cli-mvp-core-finalization]: Init repairs constitution state through an internal helper instead of the public update path.
- [Phase 01-cli-mvp-core-finalization]: Reserved constitution validation is centralized for mutable user input while keeping the stored constitution system-managed.
- [Phase 01-cli-mvp-core-finalization]: Expose expected_revision in CLI update JSON while preserving expected_head as a backward-compatible alias.
- [Phase 01-cli-mvp-core-finalization]: Return structured JSON error objects from the CLI instead of plain strings.
- [Phase 01-cli-mvp-core-finalization]: Keep README query examples explicit about v1 extension-search deferral.
- [Phase 02-core-test-harness-validation]: Exercise init/query behavior through the real Postgres adapter instead of mocks or CLI indirection.
- [Phase 04]: Use dedicated verification reports for completed phases instead of relying on SUMMARY-side self-checks.
- [Phase 04]: Lock the CLI's structured error envelope with a real black-box stderr regression and golden fixture.
- [Phase 04]: Mark the milestone audit as passed once the verification evidence is explicit and traceable.
- [Milestone v1.1]: Add MCP as a separate Rust interface crate instead of expanding the CLI crate beyond its role.
- [Milestone v1.1]: Reuse `docracy_core` service functions for MCP tools so the business contract stays single-sourced.
- [Milestone v1.1]: Support stdio and Streamable HTTP from one handler stack, driven by OpenCode and OpenWebUI compatibility needs.
- [Milestone v1.1]: Keep v1.1 focused on MCP tools and defer OAuth/prompts/resources until the base interface is proven.
- [Phase 01-mcp-crate-interface-boundary]: Keep crates/mcp library-first and transport-agnostic; transports wrap shared bootstrap
- [Phase 01-mcp-crate-interface-boundary]: Own runtime/config in docracy-mcp; delegate business rules to docracy-core use-cases
- [Phase 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions]: Model the repo-owned instructions document as governance everywhere in core while preserving the on-disk bundle layout.
- [Phase 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions]: Lock CLI and MCP startup to a fixed repo-owned ./governance bundle helper instead of passing a path override through startup config.
- [Phase 06-craft-launch-marketing-plan]: Position Docracy as durable, versioned document storage for agents instead of a generic notes app or vector database.
- [Phase 06-craft-launch-marketing-plan]: Lead launch messaging with Postgres-backed document storage, revision history, and repo-owned governance.
- [Phase 07-custom-sql-query-strings]: Use a typed QueryExecution enum so raw SQL and guided parsing stay explicit at the core boundary.
- [Phase 07-custom-sql-query-strings]: Default repository raw-query support returns an unsupported-storage error unless an adapter overrides it.
- [Phase 07-custom-sql-query-strings]: Relax async-trait futures to ?Send for repository object safety across the core and adapters.
- [Phase 07-custom-sql-query-strings]: Raw SQL takes precedence over guided query fields when `sql` is present.
- [Phase 07-custom-sql-query-strings]: Raw SQL runs inside a read-only transaction and uses server-enforced ceilings of 100 rows and 5000ms.
- [Phase 07-custom-sql-query-strings]: Raw rows are returned as JSON maps so the adapter never guesses column types.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Use a reserved global workspace row and shared fallback for unbound sessions so governance stays readable.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Bind workspace identity with PgPool after_connect set_config('docracy.workspace_id', ...) on each pooled connection.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Enforce workspace-consistent document/revision lineage with composite foreign keys and RLS forced on both tables.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Bind tenant scope from WORKSPACE_ID at process startup and keep missing values on the shared/global fallback path.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Store the bound workspace on McpRuntime so the process lifetime retains the session scope alongside the scoped repository.
- [Phase 08-workspace-tenancy-via-mcp-session-binding-and-postgres-rls-isolation]: Use project-scoped OpenCode env substitution instead of repository-path heuristics for client selection.
- [Phase 09-cli-workspace-create-command]: Keep workspace provisioning CLI-only and return the created UUID as JSON on stdout
- [Phase 09-cli-workspace-create-command]: Reject malformed and nil workspace IDs before connecting to Postgres so invalid input fails fast with structured validation errors
- [Phase 09-cli-workspace-create-command]: Persist workspaces through PgRepository instead of embedding raw SQL in the CLI
- [Phase 10-task-scoped-init-contexts]: Preserve the existing init contract by keeping context_documents as the full active context set and adding task_context_documents as an opt-in subset.
- [Phase 10-task-scoped-init-contexts]: Use extensions.task_scopes as the only task selector so specialty init contexts stay data-driven and do not require a new tool surface.
- [Phase 10-task-scoped-init-contexts]: Keep context_documents as the full active context set and add task-scoped fields alongside it.
- [Phase 10-task-scoped-init-contexts]: Use DOCRACY_TASK_SCOPE as the only CLI input for specialty init selection.
- [Phase 10-task-scoped-init-contexts]: Use extensions.task_scopes as the data-driven selector for scoped contexts.
- [Phase 10-task-scoped-init-contexts]: Keep task scope process-configured via DOCRACY_TASK_SCOPE instead of adding a tool parameter.
- [Phase 10-task-scoped-init-contexts]: Preserve context_documents as the full active set and add task_context_documents as an additive subset.
- [Phase 11]: State the current milestone as v1.1 MCP Server Interface instead of v1.0
- [Phase 11]: Document DOCRACY_TASK_SCOPE as optional and additive rather than a replacement for context_documents
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Use extensions.embedding as the opt-in payload carrier so the core document model stays unchanged.
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Keep mirror rows current-only with (workspace_id, document_id) as the unique key.
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Store embedding payloads as JSONB arrays plus an explicit dimension column for later Qdrant dispatch.
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Use workspace-scoped Qdrant collections keyed by document id so vector points overwrite cleanly instead of accumulating stale embeddings.
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Treat Postgres as canonical for filtering and hydration; Qdrant only supplies ranked ids.
- [Phase 12-vector-mirror-helper-and-vector-query-support]: Keep archive/deleted state authoritative in Postgres and mirror it into vector payloads for regression checks.
- [Phase 13]: Use EmbeddingJobRecord plus canonical JSON text so the worker receives a stable snapshot of the document payload.
- [Phase 13]: Key embedding jobs by workspace/document/model and reset pending metadata on overwrite so stale work is replaced in place.
- [Phase 13]: Keep the existing vector mirror queue path alongside the new embedding queue for compatibility with the current vector-mirror phase.

### Milestone Setup

- v1.0 remains shipped and archived with 26/26 requirements satisfied.
- v1.1 is now defined in `.planning/REQUIREMENTS.md` and `.planning/ROADMAP.md`.
- MCP research and client-compatibility notes are captured in `.planning/research/MCP_SERVER.md`.
- Phase 1 planning artifacts are captured in `.planning/phases/01-mcp-crate-interface-boundary/`.

### Roadmap Evolution

- Phase 5 added: Clean up governance model: make all governance docs type=governance, remove constitution special-case, and ensure Docracy always resolves ./governance as repo-owned instructions.
- Phase 6 added: Craft launch marketing plan.
- Phase 7 added: Custom SQL Query Strings.
- Phase 7 refined: single `sql` field, guided fallback, read-only raw-table execution, and server-enforced timeout/row ceilings.
- Phase 8 added: Workspace tenancy with generated workspace IDs, explicit MCP session binding through project-scoped client config, and Postgres RLS isolation.
- Phase 10 added: Task-scoped init contexts.
- Phase 11 added: Refresh README usage docs.
- Phase 12 added: Vector mirror helper and vector query support.

### Pending Todos

- Add specialty init context
- Package installable CLI binaries
- Design vector mirror contract
- Create GSD-style workflow doc
- Craft launch marketing plan
- Add MCP server to local opencode
- Make governance path repo-defined in opencode config
- Refresh README usage docs

### Blockers/Concerns

- `gsd-tools` phase lookup currently resolves `01` to the archived v1.0 phase directory, so future `/gsd-* phase 1` commands need a milestone-aware fix or manual path awareness.
- Plan-local requirement IDs WS-02 and WS-04 were not present in .planning/REQUIREMENTS.md, so automatic requirement marking skipped.
- DOCRACY_TEST_DATABASE_URL was not available in the local environment, so live Postgres integration verification for the vector queue skipped and must be rerun with a database URL.
- Plan requirement VEC-01 was not present in .planning/REQUIREMENTS.md, so automatic requirement marking could not update the traceability table.

## Session Continuity

Last session: 2026-04-10T23:42:25.286Z
Stopped at: Completed 13-01-PLAN.md
Resume file: None
