# Research Summary: Docracy

**Defined:** 2026-04-05

## Stack

- Rust core with tokio, serde, tracing, thiserror
- Postgres + sqlx as the source of truth
- Qdrant for semantic mirror/search
- clap for CLI, axum for service surface

## Table Stakes

- Typed document create/update/archive lifecycle
- Revision history that never overwrites prior state
- Structured retrieval by id, metadata, date, status, and relation
- Seed governance documents for core system behavior
- Core-test-first interface strategy

## Watch Out For

- Letting semantic search replace structured data
- Losing revision history on update
- Allowing metadata to become an ungoverned dumping ground
- Building interfaces before the document model is stable

## Direction

The roadmap should prioritize the Rust document engine, then retrieval and policy rules, then interfaces. The system is meant to be general-purpose infrastructure for agent memory, not a git wrapper or a programming-only tool.
