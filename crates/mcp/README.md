# Docracy MCP

This crate exposes the shipped Docracy contract (Init/Create/Read/Query/Update) over MCP.

## Workspace Binding

The MCP server can run in either shared/global mode or workspace-bound mode.

- If `WORKSPACE_ID` is set at startup, the process binds that workspace UUID for the full lifetime of the MCP session.
- If `WORKSPACE_ID` is omitted, the server stays in the shared/global path.
- Project-scoped OpenCode config should provide `WORKSPACE_ID` from client environment (for example `DOCRACY_WORKSPACE_ID`), not by inferring identity from the repository path.
- Shared/global governance remains readable in both modes; workspace-scoped sessions see their workspace plus the shared governance rows.
- Startup still uses the fixed repo-owned `./governance` bundle.

## Tools

Tool names are stable:

- `init`
- `create`
- `read`
- `query`
- `update`

The JSON shapes are intentionally aligned with the existing CLI/core contract.

### `init`

Input: none

Output:

```json
{
  "governance": { "files": [{ "name": "...", "content": "..." }] },
  "context_documents": []
}
```

### `create`

Input:

```json
{ "type": "...", "content": { "...": "..." }, "extensions": {} }
```

Output:

```json
{ "document": {}, "revision": {} }
```

### `read`

Input:

```json
{ "ids": ["..."] }
```

Output:

```json
{ "documents": [], "missing_ids": [] }
```

### `query`

Input: `docracy_core::query::QueryInput`

```json
{
  "sql": "SELECT id, \"type\", status FROM documents",
  "limit": 50,
  "timeout_ms": 5000
}
```

If `sql` is present, it takes precedence over `query`/`where`/`order_by`/`select`. Raw SQL runs read-only and is clamped to 100 rows and 5000ms.

Guided fallback example:

```json
{
  "query": "...",
  "where": {},
  "order_by": [{ "field": "...", "direction": "asc" }],
  "select": ["..."],
  "limit": 50,
  "cursor": "..."
}
```

Output: `QueryResult` serialized to JSON (same as CLI)

### `update`

Input:

```json
{
  "id": "...",
  "expected_revision": "...",
  "content": { "...": "..." },
  "extensions": {},
  "status": "..."
}
```

Note: `expected_head` is accepted as an alias for `expected_revision`.

Output:

```json
{ "document": {}, "new_revision": {}, "superseded_revision": {} }
```

## Alignment With CLI

- The tool inputs/outputs intentionally mirror the CLI JSON contract.
- `init` is a tool call, but server startup configuration (database URL, fixed repo-owned `./governance` bundle, migration policy) is handled at process startup rather than via tool parameters.
- Workspace binding is also handled at process startup through `WORKSPACE_ID`, so client config can select a tenant without adding a tool argument.

## Error Contract

Tool failures are returned as MCP `ErrorData` and include a stable machine-readable JSON payload:

```json
{ "kind": "...", "details": null }
```

`kind` matches `McpErrorKind` as `snake_case`. Some kinds provide structured `details` (for example revision-conflict expected/actual IDs).
