<!-- GSD:project-start source:PROJECT.md -->
## Project

**Docracy**

Docracy is a bureaucratic document system for agentic frameworks. It gives agents a structured, versioned place to create, update, search, and reason over long-lived documents instead of treating the filesystem as the primary memory layer.

The core is a Rust-backed document engine with history, metadata, and multiple interfaces layered on top. The first interfaces are a testing harness and CLI, with server and MCP support later.

**Core Value:** Agents can safely manage durable, queryable long-term memory without losing history, context, or control over document structure.

### Constraints

- **Architecture**: Core logic stays in Rust and remains interface-agnostic - CLI, API, and MCP are adapters
- **Product**: Document history and search must be first-class - losing revision context defeats the point
- **Scope**: The system should stay general-purpose - not limited to software projects
- **Governance**: Seed context and constitution documents exist to keep future agents aligned
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommendation
## Core Stack
### Runtime and language
- **Rust 2024** - best fit for a small, strict core with strong type boundaries
- **tokio** - async runtime for API, indexing, and background sync work
- **serde** - document and metadata serialization
- **thiserror** - explicit domain errors without boilerplate
- **tracing** - auditability matters for document changes and retrieval
### Persistence
- **Postgres + sqlx** - primary document store with transactional revision writes
- **JSONB fields** - flexible metadata and extension storage without abandoning structure
- **migrations** - schema changes should be explicit and reviewable
### Retrieval
- **Qdrant** - semantic mirror for embeddings and similarity search
- **structured SQL queries** - id, status, timestamp, type, and relation filters stay in the primary DB
### Interfaces
- **clap** - CLI first
- **axum** - HTTP API when the server surface lands
- **MCP adapter** later - only after the core contract stabilizes
### Testing
- **cargo test** - unit and integration coverage
- **proptest** - revision and metadata invariants
- **insta** - snapshotting rendered documents and query results
## What Not To Use
- Git as the source of truth - version control is useful, but not the memory system
- A file-only store - the project exists because files do not give enough structure
- An ORM-heavy stack - the core needs explicit document and revision control
- A single semantic index as the primary store - relevance search should not replace structure
## Confidence
- **Postgres as the primary store:** high
- **Qdrant as semantic mirror:** medium
- **Rust core with thin adapters:** high
- **CLI before server/MCP:** high
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
