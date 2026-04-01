# Roadmap: Docracy

## Active Milestone

### v1.1 MCP Server Interface

Deliver a dedicated Rust MCP server interface crate that wraps the existing storage-agnostic core and Postgres adapter, exposes Init/Create/Read/Query/Update without duplicating business rules, and is usable from both local subprocess clients and Streamable HTTP clients.

## Phases

- [ ] **Phase 1: MCP Crate + Interface Boundary** - Create the new interface crate, runtime/config bootstrap, and core-delegating handler boundary.
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
- [ ] 01-01: Workspace crate bootstrap + runtime/config model
- [ ] 01-02: Core delegation layer + MCP-facing error/response mapping

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
- [ ] 02-01: MCP tool registration, schemas, and handler coverage
- [ ] 02-02: Stdio transport, local smoke tests, and OpenCode local configuration

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

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. MCP Crate + Interface Boundary | 0/2 | Planned | - |
| 2. Tool Surface + Stdio Delivery | 0/2 | Planned | - |
| 3. Streamable HTTP + Client Compatibility | 0/2 | Planned | - |
| 4. Verification + Documentation Hardening | 0/2 | Planned | - |

## Archived Milestones

- ✅ **v1.0 MVP** — shipped 2026-04-05; archive: `.planning/milestones/v1.0-ROADMAP.md`
