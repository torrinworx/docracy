# Architecture

**Analysis Date:** 2026-04-05

## Pattern Overview

**Overall:** Crate-layered hexagonal architecture (ports-and-adapters)

**Key Characteristics:**
- Domain + use-cases live in a storage-agnostic core crate (`crates/core/`).
- Storage is an interface (`Repository`) defined in core and implemented by adapters (e.g. Postgres in `crates/postgres/`).
- Drivers/adapters (CLI, MCP) depend on core + a concrete repository adapter (`crates/cli/`, `crates/mcp/`).

## Layers

**Domain model (types + invariants):**
- Purpose: Represent documents/revisions and validate invariants.
- Location: `crates/core/src/`
- Contains:
  - Document types/status and validation rules: `crates/core/src/document.rs`, `crates/core/src/validation.rs`
  - Revision model: `crates/core/src/revision.rs`
  - IDs: `crates/core/src/ids.rs`
  - Core errors: `crates/core/src/errors.rs`
- Depends on: `serde`, `serde_json`, `chrono`, `uuid` (see `crates/core/Cargo.toml`).
- Used by: Core services (`crates/core/src/service.rs`), storage adapters (`crates/postgres/src/lib.rs`), CLI (`crates/cli/src/main.rs`).

**Use-cases (application services):**
- Purpose: Implement the system operations (Init/Create/Read/Query/Update) without knowing the storage backend.
- Location: `crates/core/src/service.rs`
- Contains:
  - `create_document`, `read_documents`, `query_documents`, `update_document`, `init_bundle`
  - Supporting “ports” for determinism/testability: `Clock` and `IdGenerator` (implemented by `SystemClock` and `UuidV4Generator`)
- Depends on:
  - `Repository` port: `crates/core/src/repository.rs`
  - Governance input port: `crates/core/src/governance.rs`
- Used by: CLI (`crates/cli/src/main.rs`) and Postgres integration tests (`crates/postgres/tests/postgres_integration.rs`).

**Ports (interfaces / boundaries):**
- Purpose: Define boundaries the core can call out to.
- Location:
  - Storage port: `crates/core/src/repository.rs` (`pub trait Repository`)
  - Governance port: `crates/core/src/governance.rs` (`pub trait GovernanceSource`)
- Contains:
  - Repository operations that are transactionally meaningful to the core (create/update with revisions, query, lookup, etc.).
- Used by:
  - Core services: `crates/core/src/service.rs` (accepts `&dyn Repository` / `&mut dyn Repository`)
  - Adapters: `crates/postgres/src/lib.rs` and `crates/core/src/memory.rs` (implement `Repository`)

**Adapters (infrastructure):**
- Purpose: Implement ports for specific infrastructures.

**Postgres repository adapter:**
- Location: `crates/postgres/src/lib.rs`
- Implements: `docracy_core::repository::Repository`
- Depends on: `sqlx` with postgres runtime/features (see `crates/postgres/Cargo.toml`).
- Key responsibilities:
  - Connection pooling + migrations: `PgRepository::connect`, `PgRepository::migrate`
  - Mapping DB rows to core types: `doc_row_to_core`, `rev_row_to_core`
  - Enforcing atomicity where needed via transactions:
    - `create_document_with_revision` and `update_document_with_revisions` wrap multi-table changes in a SQL transaction.
  - Query paging is keyset/cursor-based using `(timestamp, id)` comparisons: `push_query_filters` + `DocumentQueryCursor`.

**In-memory repository adapter:**
- Location: `crates/core/src/memory.rs`
- Implements: `Repository`
- Used for: unit-level tests and fast core validation without a DB.

**Drivers (primary entrypoints):**

**CLI binary (`docracy`):**
- Location: `crates/cli/src/main.rs` (declared in `crates/cli/Cargo.toml` via `[[bin]]`)
- Responsibilities:
  - Parse CLI args (`clap`) and load JSON inputs.
  - Establish DB connection and run migrations (unless `--no-migrate`).
  - Call core use-cases and serialize results as JSON.

**MCP interface crate (`docracy-mcp`):**
- Location: `crates/mcp/`
- Responsibilities:
  - Own MCP-facing configuration and runtime bootstrap (database URL, governance path, migration behavior, transport selection).
  - Initialize shared dependencies (`PgRepository`, `FsGovernanceSource`, `SystemClock`, `UuidV4Generator`) in one place so multiple transports can reuse the same startup path.
  - Map protocol-facing request/response shapes and errors without changing domain behavior.
- Explicit boundary:
  - **Business rules stay in `docracy_core`** (the canonical use-cases in `crates/core/src/service.rs`: `create_document`, `read_documents`, `query_documents`, `update_document`, `init_bundle`).
  - Put plainly: business rules stay in `docracy_core`.
  - `crates/mcp` is a thin interface/driver layer and must delegate to those core use-cases rather than reimplementing document/governance rules.
  - Transport-specific serving code (stdio / Streamable HTTP) should wrap the shared bootstrap/handler logic instead of duplicating it.

## Data Flow

**Init (governance load + context docs + constitution bootstrap):**
1. CLI constructs a governance source: `FsGovernanceSource::new(...)` (`crates/cli/src/main.rs`).
2. CLI calls `init_bundle(&mut repo, &governance, &clock, &ids)` (`crates/core/src/service.rs`).
3. Core loads governance files via `GovernanceSource::load_bundle` (filesystem impl: `crates/core/src/governance.rs`).
4. Core bootstraps or repairs the immutable constitution document via repository calls:
   - `Repository::find_latest_document_by_type`
   - `Repository::create_document_with_revision` (insert if missing)
   - `update_document(...)` (repair content/status if mismatched)
5. Core returns governance bundle + active context documents via `Repository::list_active_context_documents`.

**Create (document + first revision):**
1. CLI reads JSON into `NewDocument` (`crates/cli/src/main.rs`).
2. Core validates input + constructs `Document` and `DocumentRevision` (`crates/core/src/service.rs`).
3. Core persists atomically through `Repository::create_document_with_revision`.

**Update (new revision + supersede old revision):**
1. Core loads the document and current revision: `Repository::get_document` + `Repository::get_revision` (`crates/core/src/service.rs`).
2. Core creates a new revision, marks the prior revision as superseded, updates document pointers/timestamps.
3. Core persists via `Repository::update_document_with_revisions` to keep document + revisions consistent.

**Query (filtering + keyset pagination):**
1. CLI reads JSON into `QueryInput` (`crates/cli/src/main.rs`).
2. Core parses + validates query shape, applying defaults (e.g. active-only unless archived/deleted requested): `QueryInput::parse` (`crates/core/src/query.rs`).
3. Core delegates to `Repository::query_documents` (`crates/core/src/repository.rs`).
4. Adapter returns matching documents + `next_cursor` as `(ts,id)` keyset cursor (`crates/postgres/src/lib.rs` or `crates/core/src/memory.rs`).
5. Core projects results into selected fields: `project_rows` (`crates/core/src/query.rs`).

**State Management:**
- Core is stateless between calls; all durable state is persisted behind `Repository` (DB or memory).
- The only “runtime state” passed into use-cases is via dependency injection:
  - time: `Clock` (`SystemClock`)
  - ids: `IdGenerator` (`UuidV4Generator`)

## Key Abstractions

**Repository (storage port):**
- Purpose: Storage boundary for all durable operations.
- Definition: `crates/core/src/repository.rs`
- Implementations:
  - Postgres: `crates/postgres/src/lib.rs` (`PgRepository`)
  - In-memory: `crates/core/src/memory.rs` (`MemoryRepository`)

**GovernanceSource (input port):**
- Purpose: Source of governance bundle (markdown files), used by Init.
- Definition: `crates/core/src/governance.rs`
- Implementation:
  - Filesystem: `FsGovernanceSource` (`crates/core/src/governance.rs`)
- Seed governance docs live at: `governance/CONSTITUTION.md`

**Query model (core query language):**
- Purpose: Stable, validated query shape independent of any DB.
- Input parsing: `crates/core/src/query.rs` (`QueryInput::parse` → `DocumentQuery`)
- Cursor encoding: `encode_cursor` / `decode_cursor` (`crates/core/src/query.rs`)

## Entry Points

**CLI binary:**
- Location: `crates/cli/src/main.rs`
- Triggers: `cargo run -p docracy-cli -- ...` or installed `docracy` binary.
- Responsibilities:
  - Connect to Postgres (`PgRepository::connect` in `crates/postgres/src/lib.rs`).
  - Run migrations (`PgRepository::migrate` uses `migrations/` via `sqlx::migrate!`).
  - Execute one of: Init/Create/Read/Query/Update/Migrate.

## Storage Model (as implemented)

**Postgres schema + migrations:**
- Location: `migrations/`
- Tables:
  - `documents` and `document_revisions`: `migrations/0001_documents_and_revisions.sql`
- Notable constraints/indexes:
  - Enforces “exactly one constitution document”: partial unique index on `documents(type)` where `type='constitution'` (`migrations/0002_single_constitution.sql`).
  - Full-text search on `documents.content` via generated `tsvector` column + GIN index (`migrations/0003_content_search.sql`).

## Error Handling

**Strategy:** Typed domain/storage errors in core; CLI wraps to user-facing JSON error.

**Patterns:**
- Core defines `ValidationError`, `RepoError`, `GovernanceError`, and `CoreError`: `crates/core/src/validation.rs`, `crates/core/src/errors.rs`.
- CLI uses `anyhow::Result` and prints `{"error": "..."}` to stderr on failure: `crates/cli/src/main.rs`.
- Postgres adapter maps `sqlx::Error` into `RepoError` (conflict via SQLSTATE 23505): `map_sqlx_error` in `crates/postgres/src/lib.rs`.

## Cross-Cutting Concerns

**Validation:**
- Input + state invariants are enforced in core before persistence:
  - `Document::validate` (`crates/core/src/document.rs`)
  - `DocumentRevision::validate` (`crates/core/src/revision.rs`)
  - `QueryInput::parse` validates filter keys and defaults (`crates/core/src/query.rs`).

**Logging:** Not implemented (no `tracing`/logger wiring in `crates/cli/src/main.rs`).

**Authentication/Authorization:** Not applicable (CLI-only; no network service boundary).

---

*Architecture analysis: 2026-04-05*
