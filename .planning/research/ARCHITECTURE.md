# Architecture: Docracy

**Defined:** 2026-04-05
**Goal:** separate the durable document engine from every interface that calls it

## Component Boundaries

### Core document engine

Owns document types, content validation, revisions, lifecycle state, and metadata rules. Nothing outside the engine should mutate document state directly.

### Persistence layer

Writes documents, revisions, and indexable metadata to Postgres. This layer is responsible for transactional integrity and audit-friendly history.

### Search mirror

Consumes committed document events and updates semantic/vector indexes. It should never be the source of truth.

### Policy layer

Enforces constitution/context rules, document-type constraints, and write-time validation hooks.

### Interfaces

- **Test harness** - drives the core directly
- **CLI** - thin command surface for document operations
- **HTTP API** - self-hosted service surface
- **MCP server** - later adapter for agent tooling

## Data Flow

1. Interface receives a create/update/read request
2. Core validates the request against document rules and policy hooks
3. Persistence writes a new revision or lifecycle change transactionally
4. Search mirror updates structured and semantic indexes from the committed change
5. Read paths query the structured store first, then use semantic ranking when needed

## Build Order

1. Define the document and revision model
2. Implement persistence and lifecycle transitions
3. Add structured retrieval and metadata queries
4. Add semantic mirror synchronization
5. Layer CLI and service adapters on top

## Key Implication

If the core model is wrong, every interface is wrong. The roadmap should therefore front-load the document engine, revision rules, and retrieval semantics before any UI or network surface.
