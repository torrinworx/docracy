---
phase: 01-cli-mvp-core-finalization
verified: 2026-04-05T20:34:01Z
status: passed
score: 3/3 must-haves verified
---

# Phase 1: cli-mvp-core-finalization Verification Report

**Phase Goal:** Users can run the full CLI against Postgres while the core document model, governance seed, revision chain, and query semantics are finalized.
**Verified:** 2026-04-05T20:34:01Z
**Status:** passed
**Re-verification:** No

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Stale updates are rejected when the expected head revision does not match. | ✓ VERIFIED | `service.rs` checks `expected_head`; `memory.rs`/`postgres/src/lib.rs` reject mismatches; core tests pass. |
| 2 | Successful updates append one immutable revision and advance the document head atomically. | ✓ VERIFIED | `service.rs` builds v2 from v1; Postgres update runs in one transaction; `cargo test -p docracy-core` passes. |
| 3 | Core and Postgres repository paths agree on the same OCC behavior. | ✓ VERIFIED | Shared `Repository` trait, aligned `MemoryRepository`/`PgRepository` logic, and Postgres integration test all confirm the same conflict rule. |

**Score:** 3/3 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `crates/core/src/service.rs` | update/init orchestration | ✓ VERIFIED | Expected-head OCC, constitution reconciliation, query projection. |
| `crates/core/src/repository.rs` | shared repository contract | ✓ VERIFIED | `update_document_with_revisions` requires `expected_head`. |
| `crates/core/src/memory.rs` | in-memory adapter parity | ✓ VERIFIED | Same stale-write rule as Postgres. |
| `crates/postgres/src/lib.rs` | transactional Postgres adapter | ✓ VERIFIED | Head checked inside transaction before revision/document writes. |
| `crates/core/src/document.rs` | document validation | ✓ VERIFIED | Constitution blocked for normal input; timestamps/status validated. |
| `crates/core/src/validation.rs` | reserved-type guard | ✓ VERIFIED | Shared `ReservedConstitutionType` validation. |
| `crates/cli/src/main.rs` | CLI JSON contract | ✓ VERIFIED | `expected_revision` input, structured JSON errors, DB URL precedence. |
| `crates/core/src/query.rs` | query semantics | ✓ VERIFIED | Stable ordering, pagination, `extensions.*` rejected. |
| `README.md` | shipped examples/docs | ✓ VERIFIED | Update payload and deferred extension-search docs match code. |
| `migrations/0001_documents_and_revisions.sql` | core schema | ✓ VERIFIED | Documents + revisions constraints/indexes. |
| `migrations/0002_single_constitution.sql` | constitution uniqueness | ✓ VERIFIED | Single constitution DB invariant. |
| `migrations/0003_content_search.sql` | FTS support | ✓ VERIFIED | `content_search_tsv` + GIN index. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `update_document` | Repository OCC enforcement | expected head + write transaction | WIRED | Core rejects stale heads before composing the new revision. |
| `MemoryRepository` | `PgRepository` | same `expected_head` conflict rule | WIRED | Both adapters reject stale writes identically. |
| `init_bundle` | constitution seed | internal reconcile helper | WIRED | Init repairs/bootstrap constitution without exposing public mutation. |
| Public validation | constitution mutation | `validate_mutable_document_type` | WIRED | Normal `NewDocument`/update flows cannot create or mutate constitution docs. |
| CLI JSON input/output | core service contract | serde struct mapping | WIRED | CLI update payload, errors, and outputs match the service types. |
| README examples | shipped CLI payloads | docs updated to `expected_revision` | WIRED | Docs match the actual update/query contract. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| DOC-01 | 01-01 | Create document with stable ID + initial revision | ✓ SATISFIED | `create_document` builds revision v1 and document head. |
| DOC-02 | 01-02 | Read document by ID | ✓ SATISFIED | `read_documents` + repo `get_documents`. |
| DOC-03 | 01-01 | Metadata includes type/status/created/modified/current_revision_id | ✓ SATISFIED | `Document` struct + persistence mapping. |
| DOC-04 | 01-02 | Archive/delete lifecycle flags | ✓ SATISFIED | `Document` validation and update status handling. |
| DOC-05 | 01-02 | Extensions round-trip losslessly | ✓ SATISFIED | `extensions` persisted as JSONB and returned in reads. |
| REV-01 | 01-01 | Updates append immutable revisions | ✓ SATISFIED | `update_document` supersedes old rev and inserts new rev. |
| REV-02 | 01-01 | Historical revision readable by ID | ✓ SATISFIED | `get_revision` in both repositories. |
| REV-03 | 01-01 | Expected parent/head revision required | ✓ SATISFIED | `expected_head` checked in core + adapters. |
| REV-04 | 01-01 | Deterministic revision ordering | ✓ SATISFIED | Version/parent pointers enforced; Postgres unique `(document_id, version)`. |
| GOV-01 | 01-02 | Init returns governance seed docs | ✓ SATISFIED | `init_bundle` loads `./governance` bundle. |
| GOV-02 | 01-02 | Constitution repo-owned + immutable | ✓ SATISFIED | Internal reconcile only; normal validation blocks mutation. |
| GOV-03 | 01-02 | Init returns active context docs | ✓ SATISFIED | `list_active_context_documents` after reconcile. |
| GOV-04 | 01-02 | Interfaces prevent constitution edits | ✓ SATISFIED | Validation + update guard reject user-facing changes. |
| PG-01 | 01-01 | Postgres schema models docs/revisions with constraints | ✓ SATISFIED | `migrations/0001_documents_and_revisions.sql`. |
| PG-02 | 01-01 | Writes are transactional and atomic | ✓ SATISFIED | `PgRepository::update_document_with_revisions` uses a transaction. |
| PG-03 | 01-03 | CLI runs migrations by default | ✓ SATISFIED | CLI migrates unless `--no-migrate`; `migrate` command exists. |
| QRY-01 | 01-03 | Keyword search over content via Postgres FTS | ✓ SATISFIED | `content_search_tsv` + `websearch_to_tsquery`. |
| QRY-02 | 01-03 | Filters/orderings are stable | ✓ SATISFIED | `QueryInput` parse + deterministic sort keys. |
| QRY-03 | 01-03 | Pagination with limit and next_cursor | ✓ SATISFIED | `next_cursor` produced in core and Postgres paths. |
| QRY-04 | 01-03 | No v1 extension search | ✓ SATISFIED | `query.rs` rejects `extensions.*`; README calls it deferred. |
| CLI-01 | 01-03 | Init/Create/Read/Query/Update with JSON I/O | ✓ SATISFIED | CLI subcommands parse/emit JSON. |
| CLI-02 | 01-03 | Non-zero exit + machine-readable errors | ✓ SATISFIED | Structured JSON stderr envelope, exit 1. |
| CLI-03 | 01-03 | DATABASE_URL override support | ✓ SATISFIED | `--database-url` precedence preserved. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `README.md` | 199 | `TODO/Later` note about linked documents | info | Deferred future idea only; not part of phase 1 goal. |

### Human Verification Required

None.

### Gaps Summary

None. The phase goal is met: the CLI works against Postgres, governance seeding is rerunnable, OCC is enforced, and query/CLI semantics are finalized.

---

_Verified: 2026-04-05T20:34:01Z_
_Verifier: the agent (gsd-verifier)_
