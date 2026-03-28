# Roadmap: Docracy (v1)

## Overview

Deliver a trustworthy, Postgres-backed, versioned document store (stable document IDs + immutable revision history) with governance seeding, deterministic query/search primitives, and a CLI+tests that lock the tool contract agents depend on.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Canonical Document + Revision Store** - Durable docs+revisions in Postgres with atomic writes and OCC.
- [ ] **Phase 2: Governance Seed + Constitution Immutability** - Init seeds governance/context and prevents constitution mutation.
- [ ] **Phase 3: Deterministic Query + Keyword Search** - Stable filters/pagination plus Postgres FTS (no extensions search).
- [ ] **Phase 4: CLI Contract + Test Harness** - JSON CLI surface with machine-readable errors and locked semantics via tests.

## Phase Details

### Phase 1: Canonical Document + Revision Store
**Goal**: Users can create/read/update versioned documents safely in Postgres (immutable revision history, deterministic ordering, soft delete/archive, transactional invariants).
**Depends on**: Nothing (first phase)
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, REV-01, REV-02, REV-03, REV-04, PG-01, PG-02, PG-03
**Success Criteria** (what must be TRUE):
  1. User can create a document and read it by ID, receiving current content, metadata (`type`, `status`, timestamps, `current_revision_id`), and lossless JSON `extensions`.
  2. User can update a document by providing an expected head/parent revision; a successful update appends a new immutable revision linked to its parent and advances the document head atomically.
  3. User can read any historical revision by revision ID, and stale concurrent updates are rejected with a clear conflict error (no silent last-write-wins).
  4. User can archive/delete (soft, reversible) a document without losing history.
**Plans**: 3 plans

Plans:
- [ ] 01-01: Postgres schema + migrations enforcing invariants
- [ ] 01-02: Core create/read/update flows with immutable revision chaining
- [ ] 01-03: OCC + atomic transactions + lifecycle (archive/delete) semantics

### Phase 2: Governance Seed + Constitution Immutability
**Goal**: Users can initialize a workspace with governance seed docs (including an immutable constitution) and retrieve active context docs.
**Depends on**: Phase 1
**Requirements**: GOV-01, GOV-02, GOV-03, GOV-04
**Success Criteria** (what must be TRUE):
  1. User can run Init and receive governance seed documents sourced from `./governance` (including constitution).
  2. Init is rerunnable and results in exactly one constitution in the DB that matches the repo-owned constitution file; mismatches are surfaced clearly.
  3. Attempts to create or modify the constitution via supported interfaces are rejected.
  4. Init returns the active `context` documents from the DB, excluding archived/deleted documents.
**Plans**: 2 plans

Plans:
- [ ] 02-01: Init seeding (governance + context) with rerunnable behavior
- [ ] 02-02: Constitution immutability enforcement (API/CLI guards + DB constraints where appropriate)

### Phase 3: Deterministic Query + Keyword Search
**Goal**: Users can deterministically retrieve documents via filters, stable ordering, cursor pagination, and keyword search over content.
**Depends on**: Phase 2
**Requirements**: QRY-01, QRY-02, QRY-03, QRY-04
**Success Criteria** (what must be TRUE):
  1. User can query documents with filters on `type`, `status`, and time ranges and receive results in a stable ordering.
  2. User can paginate results with `limit` and `next_cursor` without duplicates or gaps (on a stable dataset).
  3. User can keyword-search over document content using Postgres full-text search and receive matching documents.
  4. If a user attempts to search/filter on `extensions`, the system clearly indicates it is not supported in v1.
**Plans**: 2 plans

Plans:
- [ ] 03-01: Query filters + stable ordering + cursor pagination
- [ ] 03-02: Postgres FTS keyword search over defined fields

### Phase 4: CLI Contract + Test Harness
**Goal**: Users can automate Docracy via a stable JSON CLI (Init/Create/Read/Query/Update) with machine-readable errors and a test suite that locks behavior.
**Depends on**: Phase 3
**Requirements**: CLI-01, CLI-02, CLI-03, TST-01, TST-02, TST-03
**Success Criteria** (what must be TRUE):
  1. User can run `init`, `create`, `read`, `update`, and `query` via CLI with JSON I/O.
  2. On failures (including conflicts), CLI exits non-zero and prints machine-readable error output.
  3. User can configure the CLI with `DATABASE_URL` and override it with `--database-url`.
  4. User can run unit + integration tests that cover revision invariants, migrations+Postgres persistence+init seeding, and query/search semantics (filters, ordering, pagination).
**Plans**: 2 plans

Plans:
- [ ] 04-01: CLI command surface + JSON/error contract
- [ ] 04-02: Unit + integration test harness locking semantics

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Canonical Document + Revision Store | 0/3 | Not started | - |
| 2. Governance Seed + Constitution Immutability | 0/2 | Not started | - |
| 3. Deterministic Query + Keyword Search | 0/2 | Not started | - |
| 4. CLI Contract + Test Harness | 0/2 | Not started | - |
