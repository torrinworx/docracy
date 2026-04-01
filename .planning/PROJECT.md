# Docracy

## What This Is

Docracy is a Postgres-backed document bureaucracy store for agentic frameworks. It provides typed documents with status and full revision history, plus query/search primitives, so agents can manage long-lived knowledge in a database instead of a filesystem.

## Core Value

Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).

## Current State

- v1.0 is shipped and archived on 2026-04-05.
- The CLI-backed MVP, core OCC, governance seeding, Postgres invariants, query/search, and validation harness are complete.
- Phase 4 closed the remaining audit evidence gap with dedicated verification artifacts and CLI stderr coverage.
- v1.1 is now planned around adding an MCP server interface crate alongside the existing CLI.

## Requirements

### Validated

- ✓ Versioned documents, revision history, and document metadata shipped in v1.0.
- ✓ Governance seed, constitution immutability, and active context init shipped in v1.0.
- ✓ Postgres storage, migrations, and atomic writes shipped in v1.0.
- ✓ Query/search, pagination, and deferred extension search shipped in v1.0.
- ✓ CLI JSON I/O, machine-readable errors, and database URL override shipped in v1.0.
- ✓ Direct-core tests and Postgres integration coverage shipped in v1.0.

### Active

- [ ] Add a Rust MCP server interface crate as a separate interface layer alongside the CLI.
- [ ] Expose Init/Create/Read/Query/Update over MCP without moving business rules out of `docracy_core`.
- [ ] Support stdio and Streamable HTTP transports so local OpenCode and native OpenWebUI flows are both viable.
- [ ] Add MCP-focused configuration, integration tests, and setup/troubleshooting docs.

### Out of Scope

- Vector DB parity/mirroring (future milestone) — not required for v1
- OAuth/DCR, RBAC, and multi-user hosted API concerns — separate future milestone after the first MCP interface ships
- MCP prompts/resources/sampling — tool surface first

## Context

- Filesystems are good for code but weak for documentation: no types/status/history, awkward global search, and poor concurrency for multi-agent edits.
- The system is intentionally a "document store" (arbitrary content + agent-defined extensions), not a normalized business database.
- Governance seed documents teach agents how to use the system and how to evolve the context documents over time.
- v1.0 shipped with 26/26 v1 requirements satisfied and 16 executed tasks across 7 plans.
- The next milestone is scoped to a thin MCP wrapper over the existing core/Postgres architecture, not a rewrite of the core delivery model.
- OpenCode local subprocess support and OpenWebUI Streamable HTTP support are the concrete client targets driving the interface design.

## Constraints

- **Core architecture**: Core logic is Rust library functions, storage-agnostic; interfaces (CLI/server/MCP) are layered on top.
- **Storage**: Postgres is the first backend.
- **Governance**: `constitution.md` is immutable and part of the codebase; the DB must reflect it.
- **Extensibility**: Documents have an `extensions` object; querying extensions is deferred until governance defines policies.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Store documents as revisioned records | Enables auditability and safe concurrent evolution | ✓ Good |
| Postgres first; vector DB later | Ship core doc store and query first | ✓ Good |
| Constitution is repo-owned + immutable | Prevents agents from rewriting governance | ✓ Good |
| Phase 1 finalizes the CLI-backed MVP | Keeps the first delivery focused on the user-facing contract | ✓ Good |
| Phase 2 tests the core directly | Keeps validation independent from the CLI surface | ✓ Good |
| Expected-head OCC is required on updates | Prevents stale writes from corrupting history | ✓ Good |
| CLI errors must be machine-readable JSON | Keeps automation simple and stable | ✓ Good |
| Init may repair constitution only through system-only paths | Preserves governance immutability | ✓ Good |
| Repository invariants belong in deferred DB checks when needed | Keeps transactional flows safe | ✓ Good |
| Milestone verification artifacts are first-class | Keeps audit evidence explicit and traceable | ✓ Good |
| MCP will be added as a separate interface crate | Preserves the existing storage-agnostic core + adapter layering | ✓ Planned |
| MCP handlers will reuse the current core service functions | Avoids business-rule drift between CLI and MCP | ✓ Planned |
| Target MCP transports are stdio and Streamable HTTP | Matches the current MCP spec and target clients (OpenCode/OpenWebUI) | ✓ Planned |
| v1.1 focuses on tools, not prompts/resources/OAuth | Keeps scope tight around exposing the shipped contract first | ✓ Planned |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-06 after defining v1.1 MCP Server Interface*
