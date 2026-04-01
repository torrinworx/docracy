# MCP Server Research

**Project:** Docracy
**Domain:** Rust MCP server interface layered over the existing core + Postgres adapter
**Researched:** 2026-04-06
**Confidence:** HIGH

## Executive Summary

The cleanest next step for Docracy is a dedicated Rust MCP interface crate that stays thin: it should deserialize MCP tool inputs, construct the runtime (`PgRepository`, governance source, config), call the already-shipped core service functions, and shape results/errors back into MCP responses. The current `docracy_core` surface already fits this well because the business logic lives in storage-agnostic functions such as `init_bundle`, `create_document`, `read_documents`, `query_documents`, and `update_document`.

For Rust, the current standard choice is the official `rmcp` SDK. It supports both standard MCP transports that matter here: `stdio` and Streamable HTTP. Those transports map directly to Docracy's target clients: OpenCode supports both local subprocess MCP servers and remote MCP URLs, while OpenWebUI's native MCP integration is Streamable HTTP only.

The milestone should therefore implement one handler stack with two transports, not two separate server implementations. Ship the tool surface first, keep it aligned with the existing CLI JSON contract, and explicitly defer prompts/resources/OAuth until the tool contract is stable.

## Key Findings

### 1. Rust server pattern: one handler layer, multiple transports

- The official Rust SDK is `rmcp`.
- The SDK supports server implementations over both `stdio` and Streamable HTTP.
- The right architecture is one server handler implementation that can be served over either transport, rather than transport-specific business logic.

**Implication for Docracy:**
- Add `crates/mcp` as a thin interface crate.
- Put all Docracy-specific MCP handlers there.
- Keep transport startup (`stdio` vs HTTP) as configuration around the same handler set.

### 2. The existing core boundary is already suitable for MCP

The current core exports exactly the operations the MCP surface needs:

- `init_bundle`
- `create_document`
- `read_documents`
- `query_documents`
- `update_document`

The repository boundary is already behind the `Repository` trait, and the Postgres adapter is already isolated in `docracy_postgres::PgRepository`. That means the MCP crate does not need new business rules; it only needs interface concerns:

- configuration loading
- request/response structs and schema descriptions
- error mapping from `CoreError`/storage/setup failures into MCP tool errors
- transport startup and shutdown

**Implication for Docracy:**
- Do not move command logic into the MCP crate.
- Do not fork the CLI's business behavior.
- If shared interface mapping code becomes repetitive later, extract a small shared interface helper module, but only after the MCP crate exists.

### 3. Tool surface should stay narrow and 1:1 with the shipped contract

The initial tool surface should mirror the already-shipped user contract:

- `init`
- `create`
- `read`
- `query`
- `update`

Why this is the right starting point:

- Clients like OpenCode and OpenWebUI consume tools naturally.
- Docracy already has stable JSON payloads and output shapes for these operations.
- Reusing those shapes keeps docs, tests, and user expectations aligned.

Recommended rule:

- Default to MCP tool schemas that closely match the CLI JSON payloads.
- Only introduce MCP-specific shape changes when the protocol or SDK truly requires it, and document any divergence explicitly.

### 4. Transport choices should be client-driven

#### `stdio`

- MCP spec still treats `stdio` as the baseline transport clients should support whenever possible.
- OpenCode supports local MCP servers launched from a configured command.
- `stdio` is the best first transport for local development, CI smoke tests, and single-user workstation usage.

**Recommendation:**
- Implement and verify `stdio` first.
- Ensure the server never writes non-MCP output to stdout.
- Send logs to stderr or tracing sinks only.

#### Streamable HTTP

- The current MCP spec standardizes Streamable HTTP as the network transport.
- OpenWebUI's native MCP support is Streamable HTTP only.
- OpenCode also supports remote MCP URLs, so HTTP extends usability beyond local subprocess launches.

**Recommendation:**
- Implement Streamable HTTP in the same crate after stdio is working.
- Bind to `127.0.0.1` by default.
- Follow the spec guidance around Origin validation and session/protocol headers when the HTTP transport is added.

#### Legacy SSE / HTTP+SSE

- The MCP spec has moved on from the older HTTP+SSE pattern.
- OpenWebUI documents Streamable HTTP as the native supported MCP mode.

**Recommendation:**
- Do not spend milestone scope on legacy HTTP+SSE compatibility.

### 5. Client usability requirements are concrete

#### OpenCode

OpenCode supports:

- local MCP servers via a configured `command` array
- remote MCP servers via a configured `url`
- optional environment variables and headers

**Implication for Docracy:**
- Provide a local configuration example that starts `docracy-mcp` over stdio.
- Provide a remote configuration example pointing to the HTTP endpoint.
- Keep configuration simple enough that users can switch between local subprocess and remote URL without changing tool semantics.

#### OpenWebUI

OpenWebUI's native MCP support expects:

- Streamable HTTP
- correct MCP connection type selection
- optional auth (none, bearer, OAuth variants)

**Implication for Docracy:**
- Native OpenWebUI support requires the HTTP transport.
- For the first milestone, bearer-token or no-auth local setups are sufficient; full OAuth can wait.
- Docs should call out Docker host networking details for local usage if relevant.

### 6. Configuration should separate runtime concerns from business rules

The MCP crate needs runtime configuration that the core does not:

- database URL
- governance directory
- whether to run migrations at startup
- transport mode (`stdio` or `http`)
- HTTP bind address / port
- optional auth or shared secret hooks for remote usage

Recommended pattern:

- Keep config in the interface crate.
- Allow env vars plus explicit CLI flags or subcommands for startup.
- Preserve safe defaults: localhost binding for HTTP, transport-safe logging, and explicit non-default settings for broader exposure.

### 7. Testing should verify handlers before clients

Recommended testing stack:

- handler-level tests for request validation, schema registration, and error mapping
- Postgres-backed integration tests for end-to-end tool behavior
- transport smoke tests for stdio and at least one HTTP request path
- documentation-anchored examples for OpenCode and OpenWebUI setup

Why this matters:

- Manual client testing alone is too brittle.
- Handler-level tests catch contract regressions before they become client-specific issues.
- Postgres-backed tests ensure the MCP layer really exercises the same correctness path as the CLI.

## Recommended Milestone Shape

### Phase 1: Interface boundary and crate bootstrap

- Create `crates/mcp` and add it to the workspace.
- Define config/runtime setup.
- Implement shared handler wiring that delegates to `docracy_core`.

### Phase 2: Tool contract and stdio transport

- Register Init/Create/Read/Query/Update as MCP tools.
- Lock schema and error behavior.
- Verify local OpenCode usage.

### Phase 3: Streamable HTTP and client compatibility

- Add the HTTP transport.
- Use localhost-safe defaults.
- Verify OpenWebUI and OpenCode remote configuration paths.

### Phase 4: Verification and docs hardening

- Add Postgres-backed MCP integration coverage.
- Document setup, transport tradeoffs, and troubleshooting.
- Make the new interface understandable and repeatable for contributors.

## Sources

### Primary

- Official Rust MCP SDK (`rmcp`) README: https://github.com/modelcontextprotocol/rust-sdk
- MCP SDK overview: https://modelcontextprotocol.io/docs/sdk
- MCP transports spec (2025-06-18): https://modelcontextprotocol.io/specification/2025-06-18/basic/transports
- OpenCode MCP servers docs: https://opencode.ai/docs/mcp-servers/
- OpenWebUI MCP docs: https://docs.openwebui.com/features/extensibility/mcp/

### Codebase references

- Core public surface: `crates/core/src/lib.rs`
- Core service handlers: `crates/core/src/service.rs`
- Storage boundary: `crates/core/src/repository.rs`
- Postgres adapter: `crates/postgres/src/lib.rs`
- Current CLI wrapper: `crates/cli/src/main.rs`

---
*Research completed: 2026-04-06*
*Ready for roadmap: yes*
