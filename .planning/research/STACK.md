# Stack: Docracy

**Defined:** 2026-04-05
**Domain:** agentic document bureaucracy / long-term memory

## Recommendation

Build the core in Rust and keep interfaces thin. Use Postgres as the source of truth for documents and revisions, then mirror searchable fields into a vector store for semantic retrieval.

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
