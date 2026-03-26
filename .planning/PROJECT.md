# Docracy

## What This Is

Docracy is a bureaucratic document system for agentic frameworks. It gives agents a structured, versioned place to create, update, search, and reason over long-lived documents instead of treating the filesystem as the primary memory layer.

The core is a Rust-backed document engine with history, metadata, and multiple interfaces layered on top. The first interfaces are a testing harness and CLI, with server and MCP support later.

## Core Value

Agents can safely manage durable, queryable long-term memory without losing history, context, or control over document structure.

## Requirements

### Validated

(None yet - ship to validate)

### Active

- [ ] Core document storage with typed documents, metadata, and revision history
- [ ] Structured read/search across ids, dates, status, relations, and relevance
- [ ] Document lifecycle controls for archive, supersede, and soft delete behavior
- [ ] Extensible metadata and document-type model for agent-defined schema growth
- [ ] Core interfaces for tests first, then CLI, then service and MCP access

### Out of Scope

- Git as the system of record - the project is explicitly trying to be separate from git workflows
- A programming-only tool - the system is intended to be general purpose
- Constant micro-edits to seed context documents - the governance docs should only change when meaningfully needed

## Context

The repository describes a system for storing agent-created documents with metadata, revision history, search, and governance. The README emphasizes that file systems are awkward for document bureaucracy because they lack typed status, ownership, versioning, and global search.

The design direction is an opinionated document database with a mirrored vector search layer, not a thin wrapper around git. The first implementation target is the core logic in Rust, independent from any interface.

## Constraints

- **Architecture**: Core logic stays in Rust and remains interface-agnostic - CLI, API, and MCP are adapters
- **Product**: Document history and search must be first-class - losing revision context defeats the point
- **Scope**: The system should stay general-purpose - not limited to software projects
- **Governance**: Seed context and constitution documents exist to keep future agents aligned

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust core with separate interfaces | Keeps the memory engine reusable across CLI, API, and MCP surfaces | Pending |
| Revisioned documents with structured metadata | Needed for durable agent memory and traceable updates | Pending |
| Separate search/mirror layer | Improves retrieval without making git the source of truth | Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-05 after initialization*
