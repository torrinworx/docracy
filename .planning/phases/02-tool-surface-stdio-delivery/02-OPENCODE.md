# OpenCode MCP Setup (Local Stdio)

This phase ships a stdio MCP server binary intended to be launched as a local subprocess.

## Config Snippet

Use a `command` launch so OpenCode starts the MCP server process and talks MCP over stdio.

```json
{
  "command": [
    "cargo",
    "run",
    "-p",
    "docracy-mcp",
    "--bin",
    "docracy-mcp",
    "--",
    "--governance-dir",
    "./governance"
  ],
  "env": {
    "DATABASE_URL": "postgres://postgres:postgres@localhost:5432/docracy"
  }
}
```

## Notes

- Stdio mode reserves **stdout** for MCP protocol messages.
- Logs and startup failures go to **stderr** only.
