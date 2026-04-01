---
created: 2026-04-06T00:49:44.573Z
title: Add MCP server to local opencode
area: tooling
files:
  - .planning/STATE.md
---

## Problem

The local OpenCode setup does not yet have the Docracy MCP server wired in, so the agentic workflow cannot use the server from the same environment that is already managing these planning tasks. This needs a repeatable setup path so the local OpenCode session can discover and talk to the MCP server without manual guesswork.

## Solution

Research the local OpenCode MCP configuration path, add the Docracy MCP server to it, and verify that the server is reachable from the local agent runtime. Capture any setup steps needed for future reuse so the configuration is easy to reproduce.
