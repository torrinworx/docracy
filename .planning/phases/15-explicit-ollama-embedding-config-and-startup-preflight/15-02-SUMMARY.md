---
phase: 15-explicit-ollama-embedding-config-and-startup-preflight
plan: 02
subsystem: interface
tags: [rust, cli, mcp, ollama, startup]

# Dependency graph
requires:
  - phase: 15-01
    provides: explicit model helper and worker preflight
provides:
  - CLI startup model resolution
  - MCP startup config/runtime embed model propagation
  - interface-layer preflight before serving requests
affects: [cli, mcp, startup, ollama]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Resolve embed model once at startup and pass concrete strings into embed calls

key-files:
  modified:
    - crates/cli/src/main.rs
    - crates/mcp/src/config.rs
    - crates/mcp/src/runtime.rs
    - crates/mcp/src/bin/docracy-mcp.rs
    - crates/mcp/src/tools.rs

key-decisions:
  - "Keep CLI and MCP aligned on a single startup-resolved embedding model, while still honoring per-request overrides."
  - "Let MCP bootstrap verify or pull the configured model before serving, so failures happen at startup instead of first request."

requirements-completed: [CFG-01, IDX-03]

# Metrics
duration: 0m
completed: 2026-04-12
---

# Phase 15 Plan 02 Summary

CLI and MCP now resolve the Ollama embedding model at startup, propagate it through runtime config, and use it as the default for vector-query auto-embedding.

## Accomplishments

- Added startup-time model resolution in the CLI and MCP stdio entrypoint.
- Extended `McpStartupConfig` and `McpRuntime` with explicit Ollama configuration.
- Updated `query_vector` to use request overrides when present and the startup model otherwise.
- Added regression tests for startup fallback and override behavior.
