---
created: 2026-04-06T03:11:48.731Z
title: Make governance path repo-defined in opencode config
area: tooling
files:
  - ~/.config/opencode/opencode.json
  - .planning/phases/02-tool-surface-stdio-delivery/02-OPENCODE.md
  - crates/mcp/src/bin/docracy-mcp.rs
  - crates/mcp/src/config.rs
---

## Problem

The OpenCode MCP setup discussion treated `--governance-dir` like a user-configurable folder, but for Docracy the governance bundle is repo-defined and specific to the project. It is the repository-owned material that tells agents how to use Docracy, so the integration should not frame governance as an arbitrary local path for normal setup.

## Solution

Tighten the OpenCode and MCP setup story so the governance path is treated as repo-defined rather than user-configurable. Review whether the MCP binary should keep accepting `--governance-dir` at all, or whether the integration/docs should instead hardcode the repository governance location for local use. Update the OpenCode config guidance and any affected startup/config surfaces so they reflect Docracy's governance model.
