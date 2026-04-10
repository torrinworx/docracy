# Requirements: Docracy

**Defined:** 2026-04-06
**Milestone:** v1.1 MCP Server Interface
**Core Value:** Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).

## v1.1 Requirements

### Interface Layer

- [x] **IFC-01**: Workspace includes a separate Rust `crates/mcp` interface crate alongside `crates/cli`, preserving the storage-agnostic core + Postgres adapter layering.
- [x] **IFC-02**: MCP handlers call existing `docracy_core` use-cases (`init_bundle`, `create_document`, `read_documents`, `query_documents`, `update_document`) instead of duplicating business rules.
- [x] **IFC-03**: Request parsing, response shaping, and error mapping for MCP live in the interface layer only; domain invariants remain enforced by the core and repository adapter.

### MCP Tool Contract

- [ ] **TOOL-01**: The MCP server exposes Init/Create/Read/Query/Update as tools with machine-readable JSON schemas.
- [x] **TOOL-02**: MCP tool payloads stay aligned with the current CLI/core JSON semantics unless a deliberate interface-level difference is documented.
- [x] **TOOL-03**: MCP tool failures return stable error kinds/details suitable for automated clients.
- [ ] **TOOL-04**: Governance and constitution protections remain enforced through the same core paths already used by the CLI.

### Transports & Client Compatibility

- [ ] **TRN-01**: The MCP server supports stdio transport for subprocess-based local clients.
- [ ] **TRN-02**: The MCP server supports Streamable HTTP transport for web/remote clients; legacy HTTP+SSE is not required.
- [ ] **TRN-03**: OpenCode can use the server through a documented local `command` configuration and a documented remote `url` configuration.
- [ ] **TRN-04**: OpenWebUI can use the server through documented Streamable HTTP configuration.

### Configuration & Operations

- [x] **CFG-01**: Server configuration covers Postgres connection, governance path, migration behavior, and transport selection without hardcoded environment assumptions.
- [ ] **CFG-02**: HTTP mode binds `127.0.0.1` by default and documents origin/auth expectations for non-local deployments.
- [ ] **CFG-03**: Transport-safe logging/output rules are preserved: stdio stdout is reserved for MCP messages, and logs go to stderr/tracing sinks.

### Workspace Lifecycle

- [x] **WS-05**: Operators can create a workspace row through the CLI and receive a UUID for later `WORKSPACE_ID` binding.
- [x] **WS-06**: Workspace creation uses a generated UUID by default, accepts an explicit UUID for scripted provisioning, and keeps the reserved nil UUID mapped to the shared global workspace only.
- [x] **WS-07**: Workspace provisioning stays CLI-only; the MCP tool surface remains `Init/Create/Read/Query/Update` and does not gain workspace management tools.

### Testing & Documentation

- [x] **TST-01**: Automated tests cover MCP tool schemas, handler behavior, and error mapping without relying solely on manual client testing.
- [ ] **TST-02**: Integration tests exercise real Postgres-backed MCP flows over at least one transport.
- [x] **DOC-01**: Documentation shows how to run the MCP server locally, configure OpenCode/OpenWebUI, and troubleshoot common setup issues.
- [x] **DOC-02**: Documentation explains the interface boundary so future API/MCP work does not pull business rules out of the core.

### Governance Cleanup

- [x] **GOV-05**: Init and persistence use document type `governance` for the repo-owned instructions doc instead of `constitution`.
- [x] **GOV-06**: User-facing validation and database rules no longer special-case constitution; only the governance doc type remains reserved for repo-owned instructions.
- [x] **GOV-07**: CLI and MCP startup always load the repo-owned `./governance` bundle and do not accept arbitrary governance-directory overrides.
- [x] **DOC-03**: Public docs explain the governance document type and the fixed `./governance` startup path.

### Vector Mirroring

- [x] **VEC-01**: Workspace-scoped vector mirroring and hybrid retrieval keep Postgres as source of truth while mirroring current document revisions into Qdrant for vector search.

## Deferred / Out of Scope for v1.1

Explicitly excluded to keep the milestone focused on the first MCP interface layer.

| Feature | Reason |
|---------|--------|
| OAuth / Dynamic Client Registration | Useful later, but not required to make the server usable by local clients, OpenCode remote headers, or OpenWebUI bearer/none modes. |
| MCP prompts, resources, sampling, roots, subscriptions | Docracy's current contract is tool-oriented (`Init/Create/Read/Query/Update`); expand after the base tool interface is stable. |
| Legacy HTTP+SSE transport | The current MCP spec standardizes stdio and Streamable HTTP; OpenWebUI's native support is Streamable HTTP only. |
| Multi-user authz / RBAC | Separate security milestone; current scope is a thin interface over the existing single-database core. |
| New document business operations beyond `Init/Create/Read/Query/Update` | The milestone is about exposing the shipped contract through MCP, not broadening the domain surface. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| IFC-01 | Phase 1 | Complete |
| IFC-02 | Phase 1 | Complete |
| IFC-03 | Phase 1 | Complete |
| TOOL-01 | Phase 2 | Planned |
| TOOL-02 | Phase 2 | Planned |
| TOOL-03 | Phase 2 | Planned |
| TOOL-04 | Phase 2 | Planned |
| TRN-01 | Phase 2 | Planned |
| TRN-02 | Phase 3 | Planned |
| TRN-03 | Phase 3 | Planned |
| TRN-04 | Phase 3 | Planned |
| CFG-01 | Phase 1 | Complete |
| CFG-02 | Phase 3 | Planned |
| CFG-03 | Phase 2 | Planned |
| WS-05 | Phase 9 | Planned |
| WS-06 | Phase 9 | Planned |
| WS-07 | Phase 9 | Planned |
| TST-01 | Phase 2 | Planned |
| TST-02 | Phase 4 | Planned |
| DOC-01 | Phase 4 | Planned |
| DOC-02 | Phase 1 | Complete |
| GOV-05 | Phase 5 | Planned |
| GOV-06 | Phase 5 | Planned |
| GOV-07 | Phase 5 | Planned |
| DOC-03 | Phase 5 | Planned |
| VEC-01 | Phase 12 | Complete |

**Coverage:**
- v1.1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

---
*Requirements defined: 2026-04-06*
*Last updated: 2026-04-10 after phase 12*
