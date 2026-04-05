# Docracy MCP

This crate exposes the shipped Docracy contract (Init/Create/Read/Query/Update) over MCP.

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

Input (matches `docracy_core::query::QueryInput`):

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

## Error Contract

Tool failures are returned as MCP `ErrorData` and include a stable machine-readable JSON payload:

```json
{ "kind": "...", "details": null }
```

`kind` matches `McpErrorKind` as `snake_case`. Some kinds provide structured `details` (for example revision-conflict expected/actual IDs).
