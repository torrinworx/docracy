---
created: 2026-04-09T04:20:12.696Z
title: Refresh README usage docs
area: docs
files:
  - README.md:7-17,19-36,37-42,71-102,284-396
  - crates/mcp/README.md:1-133
---

## Problem

The main README still mixes shipped behavior, setup instructions, and older future-idea notes, so it is harder than it should be to find the current contract for MCP, CLI usage, workspaces, and context-scoped documents. The docs need a presentable pass that reflects the current system instead of the rough planning-stage framing.

## Solution

Rewrite the top-level README around the shipped contract and current usage paths, then align the MCP and workspace guidance so the CLI, OpenCode config, and context-loading behavior are described accurately and consistently.
