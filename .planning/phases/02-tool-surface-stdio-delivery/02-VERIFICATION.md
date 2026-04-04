---
phase: 02-tool-surface-stdio-delivery
verified: 2026-04-06T02:53:40Z
status: human_needed
score: 5/7 must-haves verified
human_verification:
  - test: "Run stdio server binary and list tools"
    expected: "With DATABASE_URL set and governance dir present, `docracy-mcp` stays running; an MCP client over stdio can `list_all_tools` and receives exactly {init, create, read, query, update}."
    why_human: "Requires a real Postgres + spawning the stdio subprocess; not verifiable by static inspection."
  - test: "Verify stdout discipline in successful stdio session"
    expected: "During successful startup + tool calls, stdout contains only MCP protocol traffic (no logs/banner/help). Any logs/errors are on stderr."
    why_human: "Static inspection covers explicit writes and failure-path regression test, but cannot prove third-party/runtime output behavior in a real session."
  - test: "Smoke tool behavior against a real DB"
    expected: "Calling init/create/read/query/update over MCP produces JSON payloads matching CLI semantics; governance protections/constitution enforcement behave the same as CLI."
    why_human: "End-to-end behavior depends on DB state + governance bundle contents; phase intentionally defers Postgres-backed MCP integration tests to later work (TST-02)."
---

# Phase 2: Tool Surface + Stdio Delivery Verification Report

**Phase Goal:** Local clients can use Docracy through MCP over stdio, with the same operational contract as the existing CLI surface.
**Verified:** 2026-04-06T02:53:40Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Init/Create/Read/Query/Update are exposed as MCP tools with stable, machine-readable JSON schemas. | ✓ VERIFIED | `crates/mcp/src/tools.rs` defines `#[rmcp::tool_router]` with `init/create/read/query/update`; `crates/mcp/tests/tool_contract.rs` asserts tool names and checks schema properties via `schema_as_json_value()`. |
| 2 | Tool inputs/outputs match the shipped CLI/core JSON shapes (documenting deliberate divergences where transport config replaces CLI flags). | ✓ VERIFIED | Tool outputs in `crates/mcp/src/tools.rs` mirror CLI shapes in `crates/cli/src/main.rs` (init/create/read/query/update JSON keys match); divergence (startup config vs tool params) documented in `crates/mcp/README.md` “Alignment With CLI”. |
| 3 | Tool failures return stable machine-readable error kinds plus optional structured details. | ✓ VERIFIED | `crates/mcp/src/error.rs` implements `McpError::to_error_data()` producing `{kind, details}`; `crates/mcp/tests/tool_contract.rs` asserts `ErrorData.data` contains `kind` and `details`. |
| 4 | Governance/constitution protections remain enforced by calling the existing core use-cases via `crates/mcp/src/operations.rs`. | ✓ VERIFIED | Tools call `crate::operations::*_runtime` wrappers (`crates/mcp/src/tools.rs`); operations delegate to `docracy_core::{init_bundle, create_document, read_documents, query_documents, update_document}` (`crates/mcp/src/operations.rs`). |
| 5 | A local client can launch the MCP server over stdio (stdin/stdout) and discover the tool list. | ? UNCERTAIN | Stdio binary exists and serves via `rmcp::transport::stdio()` (`crates/mcp/src/bin/docracy-mcp.rs`), but no automated test exercises a real subprocess + Postgres-backed startup + tool discovery. |
| 6 | Stdio mode is transport-safe: stdout is reserved for MCP messages; non-protocol output never goes to stdout. | ? UNCERTAIN | Code routes tracing to stderr and writes setup errors to stderr-only (`crates/mcp/src/bin/docracy-mcp.rs`); regression test asserts stdout empty on a startup failure (`crates/mcp/tests/stdio_binary_smoke.rs`). Successful-session stdout discipline still needs a live check. |
| 7 | OpenCode has a documented local `command` configuration that launches the server. | ✓ VERIFIED | `.planning/phases/02-tool-surface-stdio-delivery/02-OPENCODE.md` includes a JSON snippet with `command` launching `cargo run -p docracy-mcp --bin docracy-mcp` and `DATABASE_URL` env. |

**Score:** 5/7 truths verified

## Required Artifacts (Existence • Substance • Wiring)

| Artifact | Expected | Status | Details |
|---------|----------|--------|---------|
| `crates/mcp/src/tools.rs` | rmcp ServerHandler with tool router + implementations | ✓ VERIFIED | Substantive tool router + args types + runtime delegation; referenced from `crates/mcp/src/lib.rs` and used by tests and stdio binary. |
| `crates/mcp/src/error.rs` | stable `ErrorData` kind/details envelope | ✓ VERIFIED | Implements `to_error_data()`; used by `crates/mcp/src/tools.rs` and tests. |
| `crates/mcp/tests/tool_contract.rs` | contract tests for tool list + schema + error payload | ✓ VERIFIED | Tests tool list stability, schema properties, and error payload keys; uses in-memory duplex transport. |
| `crates/mcp/README.md` | MCP tool contract documentation | ✓ VERIFIED | Contains `## Tools`, names, IO shapes, `## Alignment With CLI`, and `## Error Contract`. |
| `crates/mcp/src/bin/docracy-mcp.rs` | stdio MCP server entrypoint | ✓ VERIFIED | Boots runtime via `docracy_mcp::bootstrap`, serves via `rmcp::transport::stdio()`, stderr-only tracing, stderr setup-error envelope. |
| `crates/mcp/tests/stdio_binary_smoke.rs` | stdout-safety regression test | ✓ VERIFIED | Uses `assert_cmd` to assert `.stdout("")` on startup failure and checks stderr contains setup error envelope keys. |
| `.planning/phases/02-tool-surface-stdio-delivery/02-OPENCODE.md` | OpenCode local MCP config | ✓ VERIFIED | Documents `command` + `DATABASE_URL` and stdout/stderr notes. |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `crates/mcp/src/tools.rs` | `crates/mcp/src/operations.rs` | tool handlers delegate to runtime wrappers | ✓ WIRED | `init_bundle_runtime/create_document_runtime/read_documents_runtime/query_documents_runtime/update_document_runtime` are called from tools. |
| `crates/mcp/src/operations.rs` | `crates/core` use-cases | core business logic remains single-sourced | ✓ WIRED | Operations call `docracy_core::init_bundle/create_document/read_documents/query_documents/update_document`. |
| `crates/mcp/src/bin/docracy-mcp.rs` | `crates/mcp/src/runtime.rs` | bootstrap runtime then serve over stdio | ✓ WIRED | Binary calls `docracy_mcp::bootstrap(&config)` then `DocracyMcpServer::new(runtime).serve(rmcp::transport::stdio())`. |

## Requirements Coverage (Phase 2 IDs)

| Requirement | Source Plan | Description | Status | Evidence |
|------------|------------|-------------|--------|----------|
| TOOL-01 | 02-01 | Tools + machine-readable schemas | ✓ SATISFIED | rmcp tool router in `crates/mcp/src/tools.rs`; schema checks in `crates/mcp/tests/tool_contract.rs`. |
| TOOL-02 | 02-01 | Tool payloads aligned with CLI/core (or documented diffs) | ✓ SATISFIED | IO keys match `crates/cli/src/main.rs`; divergence explained in `crates/mcp/README.md` (“Alignment With CLI”). |
| TOOL-03 | 02-01 | Stable error kinds/details | ✓ SATISFIED | `McpErrorKind` + `to_error_data()` envelope (`crates/mcp/src/error.rs`); test asserts keys (`crates/mcp/tests/tool_contract.rs`). |
| TOOL-04 | 02-01 | Governance protections enforced via core | ✓ SATISFIED | Tools delegate to operations; operations call core use-cases (`crates/mcp/src/tools.rs`, `crates/mcp/src/operations.rs`). |
| TST-01 | 02-01 | Automated tests cover schemas/behavior/error mapping | ✓ SATISFIED | `tool_contract` + `stdio_binary_smoke` tests cover schema stability and stdout-safety envelope behavior. |
| TRN-01 | 02-02 | Supports stdio transport for local subprocess clients | ? NEEDS HUMAN | Stdio transport is implemented in `crates/mcp/src/bin/docracy-mcp.rs`, but no automated e2e subprocess+DB test verifies a real client can discover/call tools. |
| CFG-03 | 02-02 | Stdio stdout reserved for MCP messages; logs elsewhere | ? NEEDS HUMAN | Stderr-only tracing + stderr setup error envelope + failure-path stdout regression test exist; successful-session stdout discipline still needs a live check. |

**Orphaned requirements (expected by Phase 2 but not claimed in plans):** none found (all Phase 2 IDs appear in 02-01/02-02 plan frontmatter).

## Anti-Patterns Found

No obvious stub/placeholder patterns found in the Phase 2 key files (no TODO/FIXME markers; no stdout `println!` usage in the MCP crate).

## Human Verification Required

### 1) Run stdio server binary and list tools

**Test:** Start Postgres + create a governance dir, set `DATABASE_URL`, run `cargo run -p docracy-mcp --bin docracy-mcp -- --governance-dir ./governance`, connect with an MCP stdio client and call `list_all_tools`.

**Expected:** Tool list is exactly `init/create/read/query/update`; server stays running.

**Why human:** Requires real subprocess + DB.

### 2) Verify stdout discipline in a successful stdio session

**Test:** With server running, perform a tool call (e.g., `init`) while capturing stdout/stderr from the subprocess.

**Expected:** stdout contains only MCP frames; any logs/errors are on stderr.

**Why human:** Success-path output behavior cannot be conclusively proven by static inspection.

### 3) Smoke tool behavior against a real DB

**Test:** Call `init`, then `create`, then `read/query`, then `update` over MCP; compare JSON payload semantics to the CLI.

**Expected:** Payloads match CLI semantics; governance protections behave the same.

**Why human:** Requires live Postgres and governance bundle inputs.

---

_Verified: 2026-04-06T02:53:40Z_
_Verifier: gsd-verifier_
