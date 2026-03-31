# Roadmap: Docracy (v1)

## Overview

Deliver a trustworthy, Postgres-backed, versioned document store with a finalized CLI, immutable governance, deterministic query/search, and a direct-core test harness that locks the contract agents depend on.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: CLI MVP + Core Finalization** - Finalize core document/revision behavior, governance seeding, and the CLI surface. (completed 2026-04-05)
- [x] **Phase 2: Core Test Harness + Validation** - Add direct-core tests that validate the implementation without going through the CLI. (completed 2026-04-05)
- [x] **Phase 3: Stabilization + Gap Closure** - Use validation results to harden remaining issues and close gaps. (completed 2026-04-05)
- [x] **Phase 4: Audit Verification Closure** - Restore missing phase verification artifacts and add the remaining CLI E2E stderr coverage. (completed 2026-04-05)

## Phase Details

### Phase 1: CLI MVP + Core Finalization
**Goal**: Users can run the full CLI against Postgres while the core document model, governance seed, revision chain, and query semantics are finalized.
**Depends on**: Nothing (first phase)
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, REV-01, REV-02, REV-03, REV-04, GOV-01, GOV-02, GOV-03, GOV-04, PG-01, PG-02, PG-03, QRY-01, QRY-02, QRY-03, QRY-04, CLI-01, CLI-02, CLI-03
**Success Criteria** (what must be TRUE):
  1. User can create, read, update, and query documents via the CLI with stable JSON input/output.
  2. User can update a document only when providing the expected head revision; stale writes fail with a clear conflict.
  3. User can initialize governance deterministically and keep exactly one repo-owned constitution in the database.
  4. Query results are stable, paginated, and explicit about unsupported `extensions` search in v1.
**Plans**: 3 plans

Plans:
- [x] 01-01: Revision OCC + atomic document/revision updates
- [x] 01-02: Governance init + constitution immutability
- [x] 01-03: CLI contract + query/search finalization

### Phase 2: Core Test Harness + Validation
**Goal**: Users can test the core behavior directly, without going through the CLI, using unit and Postgres-backed integration coverage.
**Depends on**: Phase 1
**Requirements**: TST-01, TST-02, TST-03
**Success Criteria** (what must be TRUE):
  1. Unit tests exercise core document/revision invariants directly through `docracy_core`.
  2. Integration tests exercise migrations, Postgres persistence, and init seeding without the CLI.
  3. Query/search semantics are locked through core-level assertions and fixtures.
**Plans**: 2 plans

Plans:
- [x] 02-01: Core test harness scaffolding
- [x] 02-02: Postgres integration coverage for core flows

### Phase 3: Stabilization + Gap Closure
**Goal**: The implementation is hardened against issues found during validation, and remaining rough edges are cleaned up.
**Depends on**: Phase 2
**Requirements**: None new; this phase is driven by validation findings.
**Success Criteria** (what must be TRUE):
  1. Validation gaps are either fixed in code or codified in tests.
  2. The core behavior remains stable across repeated test runs.
  3. No known implementation blockers remain for the CLI-backed core MVP.
**Plans**: 1 plan

Plans:
- [x] 03-01: Integrity hardening, migrate fix, and docs alignment

### Phase 4: Audit Verification Closure
**Goal**: The milestone has explicit verification artifacts for every completed phase, and the CLI structured-error path has end-to-end coverage.
**Depends on**: Phase 3
**Requirements**: None new; this phase closes audit evidence gaps.
**Success Criteria** (what must be TRUE):
  1. Completed phases 2 and 3 each have a dedicated `VERIFICATION.md` artifact.
  2. The CLI structured JSON error path has end-to-end coverage.
  3. Milestone audit evidence is complete and no longer reports verification-only gaps.
**Plans**: 1 plan

Plans:
- [x] 04-01: Verification evidence and CLI E2E coverage

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. CLI MVP + Core Finalization | 3/3 | Complete   | 2026-04-05 |
| 2. Core Test Harness + Validation | 2/2 | Complete   | 2026-04-05 |
| 3. Stabilization + Gap Closure | 1/1 | Complete   | 2026-04-05 |
| 4. Audit Verification Closure | 1/1 | Complete    | 2026-04-05 |
