# Phase 1: MCP Crate + Interface Boundary - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning
**Source:** Active v1.1 roadmap + MCP server research

<domain>
## Phase Boundary

This phase creates the dedicated Rust MCP interface crate and the shared runtime boundary it needs: workspace wiring, startup configuration, reusable runtime/bootstrap code, and a thin core-delegating service layer that later stdio and Streamable HTTP transports can share.

It does not need to finish the end-user transport story yet. Tool registration, stdio delivery, and HTTP delivery remain later phases.

</domain>

<decisions>
## Implementation Decisions

### Interface layering
- `crates/mcp` must be a separate interface crate alongside `crates/cli`.
- Business rules stay in `docracy_core`; the MCP crate is an adapter layer, not a second application core.

### Core delegation
- MCP-facing operations should call the existing exported core use-cases: `init_bundle`, `create_document`, `read_documents`, `query_documents`, and `update_document`.
- If the MCP layer needs additional wiring helpers, add them in the interface crate first; only extend the core public surface when a real reuse boundary is missing.

### Runtime and configuration
- Runtime concerns live in the MCP crate: Postgres connection setup, governance path, migration behavior, and transport selection/config.
- Runtime/bootstrap code should be transport-agnostic so stdio and HTTP can share one initialization path.

### Interface contract
- Error mapping and response shaping belong in the MCP crate.
- Core/domain error types should be translated into MCP-friendly error kinds/details without changing domain behavior.

### the agent's Discretion
- Exact module names inside `crates/mcp`.
- Whether startup helpers are organized as `config` + `runtime` modules or a nearby equivalent.
- The exact internal response/error wrapper names, as long as they stay thin and reusable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning
- `.planning/PROJECT.md` — active milestone scope and architecture constraints
- `.planning/ROADMAP.md` — phase goals, plan split, and success criteria
- `.planning/REQUIREMENTS.md` — v1.1 requirement IDs mapped to this phase
- `.planning/STATE.md` — current milestone state and prior decisions
- `.planning/research/MCP_SERVER.md` — researched MCP server approach and target clients

### Existing architecture and codebase shape
- `.planning/codebase/ARCHITECTURE.md` — current core/adapter layering to preserve
- `.planning/codebase/STRUCTURE.md` — workspace layout and where new crates belong

### Core and adapters
- `Cargo.toml` — current workspace members
- `crates/core/src/lib.rs` — public core surface already exported to interface crates
- `crates/core/src/service.rs` — canonical business use-cases the MCP layer must reuse
- `crates/core/src/errors.rs` — typed error contracts to map at the interface boundary
- `crates/core/src/governance.rs` — filesystem governance source used by init flows
- `crates/postgres/src/lib.rs` — Postgres adapter setup and migrations
- `crates/cli/src/main.rs` — current thin interface wrapper pattern and JSON error mapping

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docracy_core::{init_bundle, create_document, read_documents, query_documents, update_document}` already exposes the operations the MCP layer needs.
- `docracy_postgres::PgRepository` already owns connection and migration behavior.
- `FsGovernanceSource`, `SystemClock`, and `UuidV4Generator` already cover init/runtime dependencies.

### Established Patterns
- Interface crates are expected to stay thin and delegate to the core.
- The CLI currently performs runtime setup first, then calls core functions and maps errors at the boundary.

### Integration Points
- The new crate belongs under `crates/mcp/` and must be added to the workspace root `Cargo.toml`.
- The MCP layer should become a sibling to `crates/cli`, not a dependency inversion that pulls adapter logic into core.

</code_context>

<specifics>
## Specific Ideas

- Define a reusable server config type that includes `database_url`, `governance_dir`, `run_migrations`, and a transport enum covering at least `stdio` and `http`.
- Create a runtime/bootstrap helper that returns fully initialized dependencies instead of reconnecting separately inside each future tool handler.
- Keep error mapping structured enough to preserve conflict details such as expected vs actual revision heads.

</specifics>

<deferred>
## Deferred Ideas

- Registering MCP tools and schemas — Phase 2
- Stdio server startup and local OpenCode smoke testing — Phase 2
- Streamable HTTP transport and OpenWebUI/OpenCode remote compatibility — Phase 3
- User-facing setup and troubleshooting docs — Phase 4

</deferred>

---

*Phase: 01-mcp-crate-interface-boundary*
*Context gathered: 2026-04-06 via roadmap + MCP research*
