---
phase: 15-explicit-ollama-embedding-config-and-startup-preflight
plan: 01
subsystem: postgres
tags: [rust, postgres, ollama, embeddings, worker]

# Dependency graph
requires:
  - phase: 14-split-query-into-postgres-only-add-query-vector-with-auto-embedding-and-qdrant-options
    provides: explicit query-vector and embedding-helper split
provides:
  - explicit Ollama embed-model helper
  - repository-owned embedding model on PgRepository
  - worker startup verify-or-pull preflight
affects: [postgres, indexer, qdrant, ollama]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Explicit embed-model config passed into adapter/runtime startup

key-files:
  modified:
    - crates/postgres/src/vector.rs
    - crates/postgres/src/lib.rs
    - crates/postgres/src/indexer.rs
    - crates/postgres/tests/embedding_queue_integration.rs
    - crates/postgres/tests/indexer_integration.rs
    - crates/postgres/tests/postgres_integration.rs

key-decisions:
  - "Remove hidden Ollama embedding-model fallback and require startup config to provide the model explicitly."
  - "Run a verify-or-pull preflight before the worker starts polling, so missing models fail fast or self-heal."

requirements-completed: [CFG-01, IDX-03]

# Metrics
duration: 0m
completed: 2026-04-12
---

# Phase 15 Plan 01 Summary

Explicit Ollama model selection now lives in the Postgres adapter and worker startup path instead of being guessed inside helper code.

## Accomplishments

- Added `require_ollama_embed_model` and `verify_or_pull_ollama_embed_model` helpers.
- Changed `ollama_embed_text` to require an explicit model string.
- Added `ollama_embed_model` to `PgRepository` and used it for embedding-job enqueue payloads.
- Added worker startup preflight coverage and explicit-model integration tests.
