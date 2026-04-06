<!-- General Instructions -->
Run the following command in the console when you have finalized your response to the user to notify them:
```bash
notify-send "OpenCode Agent" "<message>"
```
Only send this after a long running context without their input like multi agent workflows or large processes running in the Build mode.

<!-- DOCRACY -->
Before responding to the user or conducting a task, run the docracy_init tool call. This will provide you with the necessary context managed by the Docracy system to operate effectively in this repository.


<!-- GSD:project-start source:PROJECT.md -->
## Project

**Docracy**

Docracy is a Postgres-backed document bureaucracy store for agentic frameworks. It provides typed documents with status and full revision history, plus query/search primitives, so agents can manage long-lived knowledge in a database instead of a filesystem.

**Core Value:** Agents can reliably store, evolve, and retrieve durable project knowledge as versioned documents via simple tools (Init/Create/Read/Query/Update).

### Constraints

- **Core architecture**: Core logic is Rust library functions, storage-agnostic; interfaces (CLI/server/MCP) are layered on top.
- **Storage**: Postgres is the first backend.
- **Governance**: `constitution.md` is immutable and part of the codebase; the DB must reflect it.
- **Extensibility**: Documents have an `extensions` object; querying extensions is deferred until governance defines policies.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (Edition 2024) | 1.85.0 | Core, storage-agnostic library + CLI implementation | **(HIGH)** Rust gives predictable performance + strong typing for document invariants (revision chaining, status/state machines). The 2024 edition is stabilized in Rust 1.85.0, which makes it a safe “modern baseline” for new Rust projects. |
| PostgreSQL | 17.0+ (17.0 release: 2024-09-26) | Durable persistence for documents + revisions + search indexes | **(HIGH)** Postgres is the “default durable store” for anything requiring transactions + auditability. Postgres 17 is a 2025-standard baseline and has mature full-text search + JSON + indexing features you need without introducing extra infra. |
| SQLx | 0.8.6 | Async Postgres access, migrations, and (optionally) compile-time checked SQL | **(HIGH)** SQLx is explicitly *not* an ORM: you keep real SQL (good for complex search + revision queries) while still getting strong type checking via `query!` / `query_as!` when you want it. Includes migrations support and `QueryBuilder` for dynamic filtering.
| Tokio | 1.50.0 | Async runtime for SQLx + CLI commands that do I/O | **(HIGH)** Tokio is the ecosystem default; SQLx supports it directly and most supporting crates assume it.
| clap | 4.6.0 | CLI command parsing (`init/create/read/update/query`) | **(HIGH)** clap v4 is the standard Rust CLI stack: good UX, shell completions, and derive-based ergonomics.
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | Serialize/deserialize typed documents | Always (typed document payloads, CLI I/O).
| serde_json | 1.0.149 | JSON handling (`content` and `extensions` stored as JSONB) | Always (JSONB is the simplest extensibility mechanism for “agent-defined” document content).
| uuid | 1.23.0 | Stable IDs (recommend UUIDv7 for time-sortable keys) | Always for document IDs + revision IDs; enable `v7` for better locality/sorting.
| time | 0.3.47 | Timestamps (`created_at`, `updated_at`) | Use instead of `chrono` unless a dependency forces `chrono`.
| tracing | 0.1.44 | Structured logs for CLI + library | Always; makes debugging agent/CLI behavior much easier than printf logging.
| tracing-subscriber | 0.3.23 | Log formatting + env-based filtering | Always in the CLI binary (library crates should not initialize global subscribers).
| anyhow | 1.0.102 | Ergonomic error handling at the CLI boundary | Use in binaries / integration tests (top-level command errors).
| thiserror | 2.0.18 | Domain + storage error enums | Use in libraries so error types stay stable and composable.
| dotenvy | 0.15.7 | `.env` support for local dev (DATABASE_URL) | Use in dev + tests; keep prod config in env/flags.
| sqlx-cli | 0.8.6 | Migrations + SQLx offline query checking workflow | Use in CI/dev to run migrations and (optionally) cache query metadata.
| testcontainers | 0.27.2 | Hermetic Postgres integration tests | Use when you want “`cargo test` just works” without requiring a developer-managed local Postgres.
| assert_cmd | 2.2.0 | CLI black-box tests | Use for end-to-end CLI behavior tests.
| predicates | 3.1.4 | Assertions for CLI stdout/stderr | Use with `assert_cmd` for readable test expectations.
| insta | 1.47.2 | Snapshot testing for CLI outputs / query results | Use when output stability matters (e.g., `query --json` / `query --table`).
| tempfile | 3.27.0 | Temp dirs for CLI + integration tests | Use for `init` tests and any “writes a repo layout” fixtures.
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-nextest | 0.9.132 | Fast, reliable test runner | Use for CI speed + isolation; run doctests separately (nextest doesn’t support them on stable Rust per its docs).
| rustfmt + clippy | (bundled with Rust) | Formatting + linting | Enforce in CI; treat clippy warnings as deny for the core crate.
| Docker (for tests) | N/A | Runs Postgres for integration tests | Required if using `testcontainers`; otherwise provide `docker compose` as a fallback.
## Installation
# Core
# Supporting
# Dev dependencies
# Tooling
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| SQLx | Diesel (+ diesel-async) | If you strongly prefer an ORM/DSL and are willing to contort search queries into a DSL; otherwise SQL-first is better for FTS + revision queries. **(MEDIUM)** |
| Postgres full-text search (FTS) + GIN | Tantivy / Meilisearch / Elasticsearch | If you need advanced relevance tuning, synonyms pipelines, or distributed search. For v1, Postgres FTS is simpler and “good enough” while keeping a single source of truth. **(HIGH for deferring, MEDIUM for long-term)** |
| Postgres extensions `unaccent` + `pg_trgm` | `ILIKE '%...%'` everywhere | Only acceptable at very small scale. Prefer indexed FTS + trigram indexes for interactive query UX. **(HIGH)** |
| testcontainers | Local Postgres + manual setup | Use manual setup only if your team cannot run Docker in CI/dev environments. **(MEDIUM)** |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `dotenv` crate | SQLx explicitly notes `dotenv` appears abandoned; using it is a maintenance risk. | `dotenvy` (0.15.7). |
| Shipping v1 on SQLite | You need concurrency, transactional guarantees, and real indexing/search. SQLite is great for local apps but will constrain versioning + concurrent agent edits. | Postgres 17+.
| A separate vector DB for v1 | Adds infra + sync complexity without validating the core “versioned documents + query/search” loop. | Postgres FTS now; add embeddings/vector later.
| `LIKE '%term%'` without indexes | Turns search into full scans; performance collapses as docs grow. | FTS (`tsvector` + GIN) + optional `pg_trgm`.
## Stack Patterns by Variant
- Use Postgres `tsvector` stored generated column + GIN index for keyword search.
- Add `unaccent` into the text search configuration for accent-insensitive queries.
- Add `pg_trgm` for partial/fuzzy matching on titles/ids (and for “did you mean” style suggestions).
- Use `websearch_to_tsquery` (never raises syntax errors) to parse search strings.
- Keep a separate “structured filters” layer (status/type/date ranges) built with SQLx `QueryBuilder` to avoid stringly SQL.
## Version Compatibility
| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| Rust 1.85.0 (Edition 2024) | sqlx 0.8.6, tokio 1.50.0, clap 4.6.0 | Pin toolchain via `rust-toolchain.toml` to avoid CI drift.
| PostgreSQL 17.x | SQLx 0.8.6 (postgres feature) | Use `tsvector` + GIN for FTS; add `pg_trgm`/`unaccent` extensions as needed.
| sqlx-cli 0.8.6 | sqlx 0.8.6 | Keep CLI and library on the same minor version.
## Sources
- Rust Edition Guide — Rust 2024, release version 1.85.0 (official): https://doc.rust-lang.org/edition-guide/rust-2024/
- Rust 1.85.0 release blog (linked from edition guide, official): https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
- PostgreSQL 17.0 release notes (official; includes release date): https://www.postgresql.org/docs/release/17.0/
- PostgreSQL full text search docs (official):
- PostgreSQL `pg_trgm` extension docs (official): https://www.postgresql.org/docs/17/pgtrgm.html
- PostgreSQL `unaccent` extension docs (official): https://www.postgresql.org/docs/17/unaccent.html
- SQLx docs (docs.rs): https://docs.rs/sqlx/latest/sqlx/ (sqlx 0.8.6)
- SQLx README (official repo; includes install guidance + notes about `dotenv` vs `dotenvy`): https://github.com/launchbadge/sqlx
- Tokio docs (docs.rs): https://docs.rs/tokio/latest/tokio/ (tokio 1.50.0)
- clap docs (docs.rs): https://docs.rs/clap/latest/clap/ (clap 4.6.0)
- serde docs (docs.rs): https://docs.rs/serde/latest/serde/ (serde 1.0.228)
- uuid docs (docs.rs; includes guidance on using v7 for sortable DB keys): https://docs.rs/uuid/latest/uuid/ (uuid 1.23.0)
- cargo-nextest latest release (official GitHub): https://github.com/nextest-rs/nextest/releases/latest (0.9.132)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
