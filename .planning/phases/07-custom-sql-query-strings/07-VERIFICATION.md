---
phase: 07-custom-sql-query-strings
verified: 2026-04-08T15:19:10Z
status: passed
score: 6/6 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 4/6
  gaps_closed:
    - "Agents can submit a single `sql` field and get raw-query results without the guided path shaping execution."
    - "The guided/raw split is locked by unit tests and `cargo test -p docracy-core --lib` passes."
  gaps_remaining: []
  regressions: []
---

# Phase 07: Custom SQL Query Strings Verification Report

**Phase Goal:** Rework query so agents can submit raw SQL directly through a single `sql` field, while preserving the existing guided path as the fallback.
**Verified:** 2026-04-08T15:19:10Z
**Status:** passed
**Re-verification:** Yes — after gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Agents can submit a single `sql` field and get raw-query results without the guided `where/order_by/select` path shaping execution. | ✓ VERIFIED | MCP `QueryArgs` now exposes `sql`/`timeout_ms` and forwards them into `QueryInput`; core/service dispatch raw mode. |
| 2 | When `sql` is absent, the existing guided query contract still behaves exactly as before. | ✓ VERIFIED | `QueryInput::parse()` still returns `Guided` with default active-only filtering, cursor encoding, and `extensions.*` rejection. |
| 3 | The core exports a typed raw-query contract so downstream code does not need to infer mode from ad hoc JSON checks. | ✓ VERIFIED | `RawQueryInput`, `RawQueryResult`, `QueryExecution`, and `QueryInput` are re-exported from `crates/core/src/lib.rs`. |
| 4 | Raw SQL queries run against Postgres in read-only mode and cannot mutate data. | ✓ VERIFIED | `PgRepository::query_raw_documents` issues `SET TRANSACTION READ ONLY`; integration test rejects `UPDATE`. |
| 5 | Caller-supplied timeout and row limits are clamped to server ceilings before execution. | ✓ VERIFIED | `raw_query_limit` clamps to 100 and `raw_query_timeout` clamps to 5000ms; integration test confirms 100-row cap. |
| 6 | The public README and MCP README explain the `sql` field, precedence rules, and ceilings. | ✓ VERIFIED | Both docs show `sql`, precedence, fallback guidance, and the 100/5000 ceilings. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/core/src/query.rs` | Raw-query contract types and parse logic | ✓ VERIFIED | Raw SQL path exists; guided defaults remain intact. |
| `crates/core/src/repository.rs` | Repository raw-query hook | ✓ VERIFIED | Default `query_raw_documents` hook exists. |
| `crates/core/src/service.rs` | Dispatch between raw and guided execution | ✓ VERIFIED | Branches on `QueryExecution` and returns stable `QueryResult`. |
| `crates/core/src/lib.rs` | Public re-exports | ✓ VERIFIED | New query-contract types are exported. |
| `crates/postgres/src/lib.rs` | Raw SQL execution and clamping | ✓ VERIFIED | Read-only transaction, ceiling clamps, JSON row wrapping. |
| `crates/postgres/tests/postgres_integration.rs` | End-to-end raw SQL coverage | ✓ VERIFIED | Covers JSON output, write rejection, and limit ceiling. |
| `README.md` | User-facing contract docs | ✓ VERIFIED | Documents raw SQL, fallback, and ceilings. |
| `crates/mcp/README.md` | MCP query docs | ✓ VERIFIED | Documents the raw SQL contract and matches the tool schema. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `QueryInput::parse` | `query_documents` | mode selection between guided and raw execution | WIRED | `service.rs` matches on `QueryExecution`. |
| `query_documents` | `Repository::query_raw_documents` | raw SQL branch | WIRED | Raw requests bypass guided projection. |
| `Repository::query_raw_documents` | `QueryResult` | JSON row materialization and stable output shape | WIRED | Raw rows are returned directly with `applied_where = {}` and `next_cursor = None`. |
| `PgRepository::query_raw_documents` | `SET TRANSACTION READ ONLY` | transaction-level read-only enforcement | WIRED | Raw SQL executes inside a read-only transaction. |
| `PgRepository::query_raw_documents` | `to_jsonb(raw_query)` | dynamic row materialization without manual column typing | WIRED | Rows are wrapped as JSON maps. |
| `README.md` | `crates/mcp/README.md` | shared query payload contract | WIRED | Both docs show the same raw SQL contract and fallback behavior. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| TOOL-02 | 07-01 | MCP tool payloads stay aligned with CLI/core JSON semantics unless a deliberate interface-level difference is documented. | SATISFIED | `crates/mcp/src/tools.rs` now forwards `sql` and `timeout_ms` into `QueryInput`, and the MCP README matches the core contract. |
| TOOL-03 | 07-01 | MCP tool failures return stable error kinds/details suitable for automated clients. | SATISFIED | `crates/mcp/src/error.rs` and `crates/mcp/tests/operations.rs` verify structured kind/details mapping. |
| TST-01 | 07-02 | Automated tests cover MCP tool schemas, handler behavior, and error mapping without relying solely on manual client testing. | SATISFIED | `crates/mcp/tests/tool_contract.rs` and `crates/mcp/tests/operations.rs` cover schema + delegation + errors. |
| DOC-01 | 07-02 | Documentation shows how to run the MCP server locally, configure OpenCode/OpenWebUI, and troubleshoot common setup issues. | SATISFIED | `README.md` and `crates/mcp/README.md` contain the query contract and startup docs. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `README.md` | 319 | `TODO/Later` note | Info | Unrelated legacy note; does not block this phase. |

### Human Verification Required

None.

### Gaps Summary

The documented raw-SQL contract is now wired end-to-end: core parsing, service dispatch, MCP query payloads, and Postgres execution all align, and the core tests compile and pass.

---

_Verified: 2026-04-08T15:19:10Z_
_Verifier: the agent (gsd-verifier)_
