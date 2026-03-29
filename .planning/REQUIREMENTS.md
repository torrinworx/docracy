# Requirements: Docracy

**Defined:** 2026-04-05
**Core Value:** Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Documents

- [x] **DOC-01**: System can create a document with a stable ID and an initial revision
- [x] **DOC-02**: System can read a document by ID and return current content, extensions, and metadata
- [x] **DOC-03**: Document metadata includes `type`, `status`, `created`, `modified`, and `current_revision_id`
- [x] **DOC-04**: Documents support lifecycle flags for archive/delete (soft delete, reversible)
- [x] **DOC-05**: Documents store and return arbitrary JSON `extensions` losslessly

### Revisions & Concurrency

- [x] **REV-01**: Every update appends an immutable revision linked to a parent revision
- [x] **REV-02**: System can read any historical revision by revision ID
- [x] **REV-03**: Update requires an expected parent/head revision and rejects stale writes (conflict)
- [x] **REV-04**: Revision ordering is deterministic (parent pointers/version counters; timestamps are metadata)

### Governance & Init

- [x] **GOV-01**: Init returns governance seed documents from `./governance` (including constitution)
- [x] **GOV-02**: Constitution is repo-owned and immutable; init ensures DB has exactly one constitution matching the repo file
- [x] **GOV-03**: Init returns active `context` documents from the DB (excluding archived/deleted)
- [x] **GOV-04**: Interfaces prevent agents from creating or modifying the constitution

### Storage & Migrations (Postgres)

- [x] **PG-01**: Postgres schema models documents + revisions with constraints enforcing core invariants
- [x] **PG-02**: Write operations are transactional and atomic (document head + revision updates)
- [x] **PG-03**: CLI runs migrations by default; schema evolution is versioned and repeatable

### Query & Search

- [x] **QRY-01**: Query supports keyword search over document content using Postgres full-text search
- [x] **QRY-02**: Query supports filters on `type`, `status`, and time ranges, with stable ordering
- [x] **QRY-03**: Query supports pagination with `limit` and `next_cursor`
- [x] **QRY-04**: Query does not support searching extensions in v1 (explicitly deferred)

### CLI

- [x] **CLI-01**: CLI exposes Init/Create/Read/Query/Update commands with JSON I/O
- [x] **CLI-02**: CLI is automatable: exits non-zero on errors and prints machine-readable errors
- [x] **CLI-03**: CLI config supports `DATABASE_URL` and a `--database-url` override

### Testing

- [ ] **TST-01**: Unit tests cover revision chaining and key invariants
- [ ] **TST-02**: Integration tests cover migrations + Postgres persistence + init seeding
- [ ] **TST-03**: Tests lock query/search semantics (filters, ordering, pagination)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Portability & Operations

- **EXP-01**: System can export/import documents + revisions as JSONL while preserving IDs
- **OPS-01**: System supports reindex/backfill commands for derived indexes (FTS now; more later)

### Multi-Workspace & Security

- **NS-01**: Documents can be scoped to a workspace/namespace for logical isolation
- **SEC-01**: Role-based access control for multi-user environments (least privilege)
- **SEC-02**: Retention/redaction strategy for secrets/PII in append-only history

### Collaboration & UX

- **DIF-01**: Diff views between revisions (text/JSON)
- **PAT-01**: Patch application creates new revisions from diffs/patches
- **CNF-01**: Conflict inspection + guided resolution workflow for concurrent edits

### Retrieval Enhancements

- **VEC-01**: Optional vector DB mirroring / hybrid retrieval (after v1 validates query patterns)
- **EXT-01**: Governance-defined extension indexing/querying policies (JSONB indexes + guardrails)

## Out of Scope

Explicitly excluded for v1 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Built-in vector DB / embeddings pipeline (v1) | Extra infra + lifecycle complexity; validate keyword + filters first |
| Real-time CRDT editor (v1) | Very high complexity and surface area; revisions + OCC first |
| LLM auto-rewrite of historical revisions | Breaks auditability and reproducibility; use append-only + derived summaries instead |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOC-01 | Phase 1 | Complete |
| DOC-02 | Phase 1 | Complete |
| DOC-03 | Phase 1 | Complete |
| DOC-04 | Phase 1 | Complete |
| DOC-05 | Phase 1 | Complete |
| REV-01 | Phase 1 | Complete |
| REV-02 | Phase 1 | Complete |
| REV-03 | Phase 1 | Complete |
| REV-04 | Phase 1 | Complete |
| GOV-01 | Phase 1 | Complete |
| GOV-02 | Phase 1 | Complete |
| GOV-03 | Phase 1 | Complete |
| GOV-04 | Phase 1 | Complete |
| PG-01 | Phase 1 | Complete |
| PG-02 | Phase 1 | Complete |
| PG-03 | Phase 1 | Complete |
| QRY-01 | Phase 1 | Complete |
| QRY-02 | Phase 1 | Complete |
| QRY-03 | Phase 1 | Complete |
| QRY-04 | Phase 1 | Complete |
| CLI-01 | Phase 1 | Complete |
| CLI-02 | Phase 1 | Complete |
| CLI-03 | Phase 1 | Complete |
| TST-01 | Phase 2 | Pending |
| TST-02 | Phase 2 | Pending |
| TST-03 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

### Notes

- Phase 3 is a stabilization loop and does not introduce new requirement IDs.

---
*Requirements defined: 2026-04-05*
*Last updated: 2026-04-05 after initial definition*
