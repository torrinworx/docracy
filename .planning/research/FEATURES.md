# Features: Docracy

**Defined:** 2026-04-05
**Core Value:** Agents can safely manage durable, queryable long-term memory without losing history, context, or control over document structure.

## Table Stakes

### Document lifecycle

- Document can be created with typed content and metadata
- Document can be updated without losing prior revisions
- Previous revisions are archived or superseded automatically
- Document can be soft-deleted or archived
- Document status is queryable

### Retrieval

- Document can be read by id
- Documents can be searched by keyword
- Documents can be searched by metadata fields
- Documents can be filtered by date, status, and type
- Related documents can be linked and traversed

### Governance

- Constitution-style policy documents can constrain updates
- Context documents can guide future agents
- Seed documents can be loaded on init

### Interfaces

- Core behavior can be exercised by tests without any UI
- CLI can create, read, update, archive, and search documents
- Service/API surface can call the same core logic later

## Differentiators

- Structured metadata fields are indexable and queryable
- Document types can evolve beyond plain text
- Revision history is first-class, not an afterthought
- Semantic mirror augments structured retrieval
- Policies can validate or block writes before persistence

## Anti-Features

- Git mirror as the primary workflow
- Freeform untyped blobs with no lifecycle rules
- Interface-specific business logic
- Requiring agents to know the full system before they can contribute

## Dependencies

- Revision history depends on the storage model
- Structured retrieval depends on metadata normalization
- Semantic search depends on a mirror/index pipeline
- Governance depends on seed document loading
