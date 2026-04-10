---
phase: 12-vector-mirror-helper-and-vector-query-support
verified: 2026-04-10T03:39:00Z
status: human_needed
score: 6/6 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 6/6
  gaps_closed:
    - "VEC-01 is now accounted for in .planning/REQUIREMENTS.md and cross-referenced to Phase 12."
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Run live vector mirroring and query flow against Qdrant"
    expected: "Postgres writes enqueue current snapshots, Qdrant flushes into workspace-scoped collections, and embedding queries return only the active workspace's current documents."
    why_human: "Requires a live Qdrant service and end-to-end runtime validation across CLI/MCP surfaces."
---

# Phase 12: Vector Mirror Helper and Vector Query Support Verification Report

**Phase Goal:** Postgres remains the source of truth while workspace-scoped vector snapshots are mirrored into Qdrant, archive/delete state stays aligned in both stores, and vector queries return only the active workspace's current documents.
**Verified:** 2026-04-10T03:39:00Z
**Status:** human_needed
**Re-verification:** Yes — after traceability gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | A document write enqueues a current mirror snapshot in the same Postgres transaction. | ✓ VERIFIED | `PgRepository::create_document_with_revision` / `update_document_with_revisions` call `enqueue_vector_mirror_snapshot` before commit; `vector_mirror_queue` has PK `(workspace_id, document_id)`. |
| 2 | Archive/delete changes overwrite the existing mirror snapshot instead of leaving stale vectors behind. | ✓ VERIFIED | `ON CONFLICT (workspace_id, document_id) DO UPDATE` refreshes the row; integration tests show one row updated in place across active → archived → active transitions. |
| 3 | Workspace-bound and global scopes never share queue rows. | ✓ VERIFIED | Queue rows use `workspace_id` from `PgRepository`, migration RLS scopes by `docracy.workspace_id`, and tests prove workspace A/B isolation. |
| 4 | Qdrant receives workspace-scoped upserts keyed by document id. | ✓ VERIFIED | `crates/postgres/src/vector.rs` builds `docracy_workspace_{workspace_id}` collections, upserts by `document_id`, and flushes pending rows only after success. |
| 5 | Vector search returns only the active workspace's current documents, with archived/deleted rows filtered like Postgres. | ✓ VERIFIED | `query_vector_documents` hydrates/search-filters through `query_documents(...)` + `get_documents(...)`; tests cover workspace scoping and archived docs disappearing. |
| 6 | Core, CLI, and MCP surfaces accept vector query input. | ✓ VERIFIED | `QueryInput.embedding` routes to `QueryExecution::Vector`; CLI reads `QueryInput` directly; MCP `QueryArgs.embedding` maps into the core query contract. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/core/src/vector.rs` | Shared mirror/query contract | ✓ VERIFIED | Defines `VectorMirrorRecord` and `VectorQueryInput`; no Qdrant client coupling. |
| `crates/core/src/query.rs` | Vector query routing | ✓ VERIFIED | `QueryInput.embedding` routes to `QueryExecution::Vector` with guided defaults preserved. |
| `crates/core/src/repository.rs` | Vector-search hook | ✓ VERIFIED | Adds `query_vector_documents` with safe unsupported-storage default. |
| `crates/core/src/service.rs` | Vector execution + hydration | ✓ VERIFIED | Dispatches vector queries, then hydrates ranked ids from Postgres before projecting rows. |
| `migrations/0007_vector_mirror_queue.sql` | Current-state mirror queue | ✓ VERIFIED | Creates `vector_mirror_queue` with workspace/document PK, RLS, and snapshot fields. |
| `crates/postgres/src/lib.rs` | Queue writes + vector search hook | ✓ VERIFIED | Enqueues mirror snapshots in-tx and implements Postgres-side vector search hydration. |
| `crates/postgres/src/vector.rs` | Qdrant dispatch helper | ✓ VERIFIED | Handles `QDRANT_URL`, workspace-scoped collection naming, flush, and search helpers. |
| `crates/postgres/tests/postgres_integration.rs` | Regression coverage | ✓ VERIFIED | Tests cover queue overwrite, workspace isolation, Qdrant failure handling, archive filtering, and query routing. |
| `crates/mcp/src/tools.rs` | MCP vector input plumbing | ✓ VERIFIED | `QueryArgs.embedding` is exposed and forwarded to core. |
| `crates/cli/src/main.rs` | CLI vector input plumbing | ✓ VERIFIED | CLI query path accepts the same JSON `QueryInput` contract. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/postgres/src/lib.rs` | `migrations/0007_vector_mirror_queue.sql` | same-transaction queue insert | WIRED | Queue row insert/update happens inside the document write transaction. |
| `crates/core/src/vector.rs` | `crates/postgres/src/lib.rs` | document snapshot extraction | WIRED | Core mirror record shape is consumed by the Postgres adapter helper. |
| `vector_mirror_queue.workspace_id` | `PgRepository.workspace_id` | session-bound scope key | WIRED | Mirror rows inherit the adapter workspace scope; migration RLS enforces it. |
| `crates/core/src/query.rs` | `crates/core/src/repository.rs` | vector query dispatch | WIRED | `QueryInput.embedding` becomes `QueryExecution::Vector`, which calls the repository hook. |
| `crates/postgres/src/vector.rs` | Qdrant collection names | workspace-scoped collection naming | WIRED | `docracy_workspace_{workspace_id}` is used for all flush/search calls. |
| `crates/postgres/src/vector.rs` | `crates/postgres/tests/postgres_integration.rs` | archive/workspace regression coverage | WIRED | Tests assert queue overwrite, isolation, and Qdrant failure behavior. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| VEC-01 | 12-01, 12-02 | Workspace-scoped vector mirroring and hybrid retrieval keep Postgres as source of truth while mirroring current document revisions into Qdrant for vector search. | SATISFIED | Present in `.planning/REQUIREMENTS.md` (line 57) and traced to Phase 12 in the registry table (line 100). |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | - | - | - | No blocker stub/placeholder anti-patterns found in the phase files. |

### Human Verification Required

1. **Live Qdrant flush and search**
   - **Test:** Write/update a document with an embedding, flush the mirror queue against a live Qdrant instance, and run an embedding query through CLI or MCP.
   - **Expected:** Queue rows are cleared after successful flush; results come only from the active workspace and reflect current Postgres state.
   - **Why human:** Requires a live Qdrant service and end-to-end runtime validation.

### Gaps Summary

The traceability gap is closed: `VEC-01` now exists in the active requirements registry and is mapped to Phase 12. The codebase still shows the full vector mirror/query wiring across core, Postgres, CLI, and MCP; remaining validation is only live-service execution against Qdrant.

---

_Verified: 2026-04-10T03:39:00Z_
_Verifier: the agent (gsd-verifier)_
