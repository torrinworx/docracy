# Roadmap: Docracy

## Active Milestone

### v1.1 MCP Server Interface

Deliver a dedicated Rust MCP server interface crate that wraps the existing storage-agnostic core and Postgres adapter, exposes Init/Create/Read/Query/Update without duplicating business rules, and is usable from both local subprocess clients and Streamable HTTP clients.

## Phases

- [x] **Phase 1: MCP Crate + Interface Boundary** - Create the new interface crate, runtime/config bootstrap, and core-delegating handler boundary.
- [ ] **Phase 2: Tool Surface + Stdio Delivery** - Expose Init/Create/Read/Query/Update as MCP tools and make the server usable from local stdio clients such as OpenCode.
- [ ] **Phase 3: Streamable HTTP + Client Compatibility** - Add the HTTP transport, safe runtime defaults, and client compatibility for OpenWebUI and remote MCP usage.
- [ ] **Phase 4: Verification + Documentation Hardening** - Lock the contract with integration coverage, examples, and contributor docs.

## Phase Details

### Phase 1: MCP Crate + Interface Boundary
**Goal**: Docracy has a dedicated Rust MCP crate that stays thin, owns runtime concerns, and delegates business logic to the existing core.
**Depends on**: v1.0 shipped baseline
**Requirements**: IFC-01, IFC-02, IFC-03, CFG-01, DOC-02
**Success Criteria** (what must be TRUE):
  1. The workspace includes a separate `crates/mcp` interface crate alongside `crates/cli`.
  2. MCP handlers call exported `docracy_core` use-cases rather than reimplementing document/governance rules.
  3. Configuration/runtime concerns are isolated in the interface layer and are reusable across multiple transports.
**Plans**: 2 plans

Plans:
- [x] 01-01: Workspace crate bootstrap + runtime/config model
- [x] 01-02: Core delegation layer + MCP-facing error/response mapping

### Phase 2: Tool Surface + Stdio Delivery
**Goal**: Local clients can use Docracy through MCP over stdio, with the same operational contract as the existing CLI surface.
**Depends on**: Phase 1
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TRN-01, CFG-03, TST-01
**Success Criteria** (what must be TRUE):
  1. Init/Create/Read/Query/Update are exposed as MCP tools with stable schemas.
  2. Stdio mode is transport-safe: stdout carries MCP messages only, while logs go elsewhere.
  3. OpenCode can launch the server locally via a documented `command` configuration.
**Plans**: 2 plans

Plans:
- [x] 02-01: MCP tool registration, schemas, and handler coverage
- [x] 02-02: Stdio transport, local smoke tests, and OpenCode local configuration

### Phase 3: Streamable HTTP + Client Compatibility
**Goal**: Docracy can also run as a Streamable HTTP MCP server for browser-adjacent and remote clients without creating a second implementation path.
**Depends on**: Phase 2
**Requirements**: TRN-02, TRN-03, TRN-04, CFG-02
**Success Criteria** (what must be TRUE):
  1. The same MCP handler set is served over Streamable HTTP.
  2. HTTP mode defaults to localhost-safe binding and documents origin/auth expectations for broader exposure.
  3. OpenWebUI can connect using its native MCP (Streamable HTTP) support, and OpenCode can use the same server remotely.
**Plans**: 2 plans

Plans:
- [ ] 03-01: Streamable HTTP transport + safe runtime defaults
- [ ] 03-02: OpenWebUI/OpenCode remote compatibility verification

### Phase 4: Verification + Documentation Hardening
**Goal**: The MCP interface is validated end-to-end and explained clearly enough for users and contributors to adopt it safely.
**Depends on**: Phase 3
**Requirements**: TST-02, DOC-01
**Success Criteria** (what must be TRUE):
  1. Postgres-backed MCP integration coverage exists for the shipped contract.
  2. Local development, OpenCode setup, and OpenWebUI setup are documented and tested against the implemented interface.
  3. Contributors can see clearly where MCP-specific code ends and core business logic begins.
**Plans**: 2 plans

Plans:
- [ ] 04-01: Postgres-backed MCP integration and transport smoke coverage
- [ ] 04-02: User docs, architecture notes, and troubleshooting guide

### Phase 7: Custom SQL Query Strings
**Goal**: Rework query so agents can submit raw SQL directly through a single `sql` field, while preserving the existing guided path as the fallback.
**Depends on**: Phase 4
**Requirements**: TOOL-02, TOOL-03, TST-01, DOC-01
**Success Criteria** (what must be TRUE):
  1. Query accepts either `sql` or guided parameters, with `sql` taking precedence when present.
  2. SQL mode is read-only only and runs against the raw database tables.
  3. Timeout and row limit are caller-requested but enforced with server-side ceilings.
  4. The query contract is documented clearly enough for agents to author SQL directly.
**Plans**: 2 plans

Plans:
- [x] 07-01: Query input contract + guided/raw mode routing
- [x] 07-02: Raw SQL execution, ceilings, docs, and integration coverage

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. MCP Crate + Interface Boundary | 2/2 | Complete | 2026-04-06 |
| 2. Tool Surface + Stdio Delivery | 0/2 | Planned | - |
| 3. Streamable HTTP + Client Compatibility | 0/2 | Planned | - |
| 4. Verification + Documentation Hardening | 0/2 | Planned | - |
| 5. Governance Model Cleanup | 2/2 | Complete   | 2026-04-06 |
| 6. Craft launch marketing plan | 1/1 | Complete | 2026-04-06 |
| 7. Custom SQL Query Strings | 0/2 | Planned | - |
| 8. Workspace tenancy via MCP session binding | 0/2 | Complete    | 2026-04-08 |
| 9. CLI Workspace Create Command | 0/1 | Planned | - |
| 10. Task-scoped init contexts | 3/3 | Complete   | 2026-04-09 |

## Archived Milestones

- ✅ **v1.0 MVP** — shipped 2026-04-05; archive: `.planning/milestones/v1.0-ROADMAP.md`

### Phase 5: Clean up governance model: make all governance docs type=governance, remove constitution special-case, and ensure Docracy always resolves ./governance as repo-owned instructions.

**Goal**: Governance is modeled as a normal repo-owned document type, existing constitution rows are migrated to governance, and CLI/MCP always resolve the fixed repo-owned `./governance` bundle.
**Requirements**: GOV-05, GOV-06, GOV-07, DOC-03
**Depends on:** Phase 4
**Plans:** 2/2 plans complete

Plans:
- [x] 05-01: Governance document type rename + persistence migration
- [x] 05-02: Fixed repo-owned governance path + documentation refresh

### Phase 6: Craft launch marketing plan

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 5
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 6 to break down)

### Phase 8: Workspace tenancy via MCP session binding and Postgres RLS isolation

**Goal**: Define workspace tenancy for Docracy with generated workspace IDs, explicit MCP session binding through project-scoped client config and `WORKSPACE_ID`, and Postgres RLS isolation so each session only sees its active workspace while shared governance stays in the global scope.
**Requirements**: WS-01, WS-02, WS-03, WS-04
**Depends on:** Phase 7
**Plans:** 2/2 plans complete

Plans:
- [x] 08-01: Workspace schema, RLS policies, and scope-aware Postgres harness
- [x] 08-02: MCP workspace binding, project config env wiring, and operator docs

### Phase 9: CLI Workspace Create Command

**Goal**: Add a CLI-only workspace management command that provisions a workspace row and returns its UUID so operators can bind `WORKSPACE_ID` manually, while leaving the MCP tool surface unchanged.
**Requirements**: WS-05, WS-06, WS-07
**Depends on:** Phase 8
**Plans:** 1 plan

Plans:
- [x] 09-01: Workspace create CLI, Postgres helper, and docs/tests

### Phase 10: Task-scoped init contexts

**Goal**: Init remains contract-preserving (returns all active `context` docs) while also returning an additive task-scoped subset derived from `extensions.task_scopes` so agents can request a specialty init context without new tools.
**Requirements**: TBD
**Depends on:** Phase 9
**Plans:** 3/3 plans complete

Plans:
- [x] 10-01-PLAN.md — Core: compute and return task-scoped context subset (without filtering active contexts)
- [x] 10-02-PLAN.md — CLI: wire DOCRACY_TASK_SCOPE and document Init output
- [x] 10-03-PLAN.md — MCP: runtime task scope + Init tool output + MCP docs

### Phase 11: Refresh README usage docs

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 10
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 11 to break down)
