# Docracy

## What This Is

Docracy is a Postgres-backed document bureaucracy store for agentic frameworks. It provides typed documents with status and full revision history, plus query/search primitives, so agents can manage long-lived knowledge in a database instead of a filesystem.

## Core Value

Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Versioned documents: type + status + revision history
- [ ] Postgres document store + migrations
- [ ] Init returns governance seed docs + active context docs
- [ ] Create/Read/Update with revision chaining, OCC, and soft delete/archive semantics
- [ ] Query with keyword search, filters, ordering, and pagination
- [ ] CLI interface finalized for the MVP
- [ ] Direct-core test harness and integration coverage
- [ ] Immutable, repo-owned constitution seeded into the DB

### Out of Scope

- Vector DB parity/mirroring (future milestone) — not required for v1
- Full hosted API/MCP server (future interface) — CLI/tests first

## Context

- Filesystems are good for code but weak for documentation: no types/status/history, awkward global search, and poor concurrency for multi-agent edits.
- The system is intentionally a "document store" (arbitrary content + agent-defined extensions), not a normalized business database.
- Governance seed documents teach agents how to use the system and how to evolve the context documents over time.

## Constraints

- **Core architecture**: Core logic is Rust library functions, storage-agnostic; interfaces (CLI/server/MCP) are layered on top.
- **Storage**: Postgres is the first backend.
- **Governance**: `constitution.md` is immutable and part of the codebase; the DB must reflect it.
- **Extensibility**: Documents have an `extensions` object; querying extensions is deferred until governance defines policies.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Store documents as revisioned records | Enables auditability and safe concurrent evolution | — Pending |
| Postgres first; vector DB later | Ship core doc store and query first | — Pending |
| Constitution is repo-owned + immutable | Prevents agents from rewriting governance | — Pending |
| Phase 1 finalizes the CLI-backed MVP | Keeps the first delivery focused on the user-facing contract | — Pending |
| Phase 2 tests the core directly | Keeps validation independent from the CLI surface | — Pending |

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
*Last updated: 2026-04-05 after initialization*
