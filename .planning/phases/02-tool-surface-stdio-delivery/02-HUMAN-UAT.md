---
status: partial
phase: 02-tool-surface-stdio-delivery
source: [02-VERIFICATION.md]
started: 2026-04-06T02:55:00Z
updated: 2026-04-06T02:55:00Z
---

## Current Test

Awaiting human testing for the live stdio subprocess flow.

## Tests

### 1. Run stdio server binary and list tools
expected: With `DATABASE_URL` set and governance dir present, `docracy-mcp` stays running; an MCP client over stdio can `list_all_tools` and receives exactly {init, create, read, query, update}.
result: [pending]

### 2. Verify stdout discipline in successful stdio session
expected: During successful startup + tool calls, stdout contains only MCP protocol traffic (no logs/banner/help). Any logs/errors are on stderr.
result: [pending]

### 3. Smoke tool behavior against a real DB
expected: Calling init/create/read/query/update over MCP produces JSON payloads matching CLI semantics; governance protections/constitution enforcement behave the same as CLI.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
