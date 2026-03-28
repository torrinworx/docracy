# Codebase Structure

**Analysis Date:** 2026-04-05

## Directory Layout

```
[project-root]/
├── crates/
│   ├── core/                 # Domain model + storage ports + use-cases (storage-agnostic)
│   ├── postgres/             # Postgres adapter implementing the core Repository port
│   └── cli/                  # Primary binary (docracy) that drives core + postgres
├── migrations/               # SQLx migrations defining Postgres schema + indexes
├── governance/               # Seed governance docs (must include CONSTITUTION.md)
├── tests/                    # Placeholder root tests directory (currently .gitkeep)
├── .planning/                # Planning artifacts for this repo/tools
├── compose.yml               # Local docker compose (present)
├── Cargo.toml                # Workspace definition
├── Cargo.lock                # Workspace lockfile
├── rust-toolchain.toml       # Toolchain components (rustfmt/clippy)
├── postgres_data/            # Local Postgres volume (gitignored)
├── qdrant_data/              # Local Qdrant volume (gitignored)
└── target/                   # Rust build output (gitignored)
```

## Directory Purposes

**`crates/core/`:**
- Purpose: Storage-agnostic core library containing domain types, validation, ports, and use-cases.
- Contains:
  - Public API re-exports: `crates/core/src/lib.rs`
  - Domain model: `crates/core/src/document.rs`, `crates/core/src/revision.rs`, `crates/core/src/ids.rs`
  - Use-cases: `crates/core/src/service.rs`
  - Query model/parsing/projection: `crates/core/src/query.rs`
  - Storage port: `crates/core/src/repository.rs`
  - Governance port: `crates/core/src/governance.rs`
  - In-memory adapter (for tests): `crates/core/src/memory.rs`
  - Error + validation types: `crates/core/src/errors.rs`, `crates/core/src/validation.rs`

**`crates/postgres/`:**
- Purpose: Concrete Postgres storage adapter.
- Contains:
  - `PgRepository` implementing `Repository`: `crates/postgres/src/lib.rs`
  - Adapter integration tests: `crates/postgres/tests/postgres_integration.rs`
- Notes:
  - Runs migrations embedded from `migrations/` via `sqlx::migrate!("../../migrations")` (`crates/postgres/src/lib.rs`).

**`crates/cli/`:**
- Purpose: Primary binary interface.
- Contains:
  - `docracy` entrypoint: `crates/cli/src/main.rs`
  - Command parsing and JSON I/O.

**`migrations/`:**
- Purpose: Postgres schema and indexing.
- Key files:
  - `migrations/0001_documents_and_revisions.sql`: base tables + constraints + indexes.
  - `migrations/0002_single_constitution.sql`: partial unique index for constitution uniqueness.
  - `migrations/0003_content_search.sql`: generated `tsvector` + GIN index for FTS.

**`governance/`:**
- Purpose: Markdown documents shipped with the binary and loaded by Init.
- Key files:
  - `governance/CONSTITUTION.md`

**`postgres_data/` + `qdrant_data/`:**
- Purpose: Local Docker volumes for dev.
- Git status: gitignored via `.gitignore` (`postgres_data/`, `qdrant_data/`).

## Key File Locations

**Entry Points:**
- `crates/cli/src/main.rs`: `docracy` CLI entrypoint (`#[tokio::main]`).

**Core API surface:**
- `crates/core/src/lib.rs`: module wiring + public re-exports used by other crates.

**Core Use-cases:**
- `crates/core/src/service.rs`: create/read/query/update/init flows.

**Storage Boundary + Adapters:**
- `crates/core/src/repository.rs`: `Repository` trait (the storage port).
- `crates/postgres/src/lib.rs`: `PgRepository` adapter.
- `crates/core/src/memory.rs`: `MemoryRepository` adapter (testing).

**Governance Boundary:**
- `crates/core/src/governance.rs`: `GovernanceSource` port + filesystem implementation `FsGovernanceSource`.

**Postgres Schema:**
- `migrations/*.sql`: DB tables/constraints/indexes referenced by the postgres adapter.

## Naming Conventions

**Rust crates:**
- Crate names use hyphenated package names in Cargo (`docracy-core`, `docracy-postgres`, `docracy-cli`) and snake-case module paths in code (`docracy_core`, `docracy_postgres`).

**Modules/files:**
- One concept per file under `crates/core/src/` (e.g. `document.rs`, `repository.rs`).

**Adapters:**
- Implement the port trait in the adapter crate root (`crates/postgres/src/lib.rs` implements `Repository`).

## Where to Add New Code

**New core operation (new “tool” / use-case):**
- Add the use-case function and related input/output structs in `crates/core/src/service.rs`.
- If it needs new storage capabilities, extend `Repository` in `crates/core/src/repository.rs` and implement the new method(s) in:
  - `crates/postgres/src/lib.rs` (Postgres adapter)
  - `crates/core/src/memory.rs` (in-memory adapter, to keep tests fast)
- Export the new use-case from `crates/core/src/lib.rs` if it is part of the public API.

**New storage backend (adapter crate):**
- Create a new crate under `crates/<backend>/`.
- Implement `docracy_core::repository::Repository` in that crate.
- Keep SQL/schema concerns out of `crates/core/`; only the port trait belongs there.

**New CLI command:**
- Add a new `Command` variant and handler in `crates/cli/src/main.rs`.
- Prefer to call core use-cases rather than embedding business logic in the CLI.

**Schema changes:**
- Add a new migration file under `migrations/` and ensure `PgRepository::migrate` continues to apply them (`crates/postgres/src/lib.rs`).

**Governance additions:**
- Add markdown files to `governance/`.
- `init_bundle` expects `CONSTITUTION.md` to exist in the governance bundle (`crates/core/src/service.rs`, `crates/core/src/governance.rs`).

## Special Directories

**`target/`:**
- Purpose: Rust build artifacts.
- Generated: Yes.
- Committed: No (`.gitignore`).

**`postgres_data/`, `qdrant_data/`:**
- Purpose: Local container volumes.
- Generated: Yes.
- Committed: No (`.gitignore`).

---

*Structure analysis: 2026-04-05*
