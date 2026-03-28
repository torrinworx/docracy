# Technology Stack

**Analysis Date:** 2026-04-05

## Languages

**Primary:**
- Rust (Edition 2021) - workspace crates in `crates/core/`, `crates/postgres/`, `crates/cli/` (see `crates/*/Cargo.toml`)

**Secondary:**
- SQL (Postgres migrations) - schema + indexes in `migrations/*.sql`

## Runtime

**Environment:**
- Rust toolchain: `stable` (pinned via `rust-toolchain.toml`)
- Async runtime: Tokio `1.51.0` (see `Cargo.lock`; used by CLI `crates/cli/src/main.rs` and tests)

**Package Manager:**
- Cargo (Rust)
- Lockfile: `Cargo.lock` (present)

## Frameworks

**Core:**
- SQLx `0.8.6` - Postgres driver + migrations + dynamic queries (`crates/postgres/src/lib.rs`, `migrations/`)
- clap `4.6.0` - CLI argument parsing (`crates/cli/src/main.rs`)

**Testing:**
- Rust built-in test harness via `cargo test`
- Tokio test macros (`#[tokio::test]`) for async tests (e.g., `crates/postgres/tests/postgres_integration.rs`, `crates/core/src/service.rs`)

**Build/Dev:**
- cargo - build/test (`Cargo.toml` workspace)
- rustfmt, clippy - installed components (`rust-toolchain.toml`)
- Docker Compose - local dependency services (`compose.yml`)

## Key Dependencies

**Critical:**
- `docracy-core` (workspace crate) - domain model + service layer (`crates/core/src/*`)
- `docracy-postgres` (workspace crate) - Postgres repository implementation (`crates/postgres/src/lib.rs`)
- serde `1.0.228` + serde_json `1.x` - JSON document `content` and `extensions` payloads (`crates/core/src/document.rs`, `crates/postgres/src/lib.rs`)
- chrono `0.4.44` - timestamps used throughout core + repo mapping (`crates/core/src/service.rs`, `crates/postgres/src/lib.rs`)
- uuid `1.23.0` - document/revision IDs (`crates/core/src/ids.rs`)

**Infrastructure:**
- async-trait `0.1.89` - async trait support for repository abstraction (`crates/core/src/repository.rs`, `crates/postgres/src/lib.rs`)
- thiserror `1.0.x` - typed error enums (`crates/core/src/errors.rs`)
- anyhow `1.0.102` - CLI error boundary (`crates/cli/src/main.rs`)
- tempfile `3.10.x` - temp dirs/files in tests (`crates/core/src/service.rs`, `crates/postgres/tests/postgres_integration.rs`)

## Configuration

**Environment:**
- Runtime DB connection:
  - `DATABASE_URL` (required if `--database-url` not provided) (`crates/cli/src/main.rs`)
- Test DB connection:
  - `DOCRACY_TEST_DATABASE_URL` (optional; falls back to `DATABASE_URL`) (`crates/postgres/tests/postgres_integration.rs`)
- Local environment files:
  - `.env` present (contains environment configuration; do not commit secrets)
  - `.env.example` present (template for environment configuration)

**Build:**
- Rust toolchain/components: `rust-toolchain.toml`
- Workspace definition: `Cargo.toml`
- SQL migrations embedded by SQLx macro: `crates/postgres/src/lib.rs` → `sqlx::migrate!("../../migrations")`

## Platform Requirements

**Development:**
- Rust stable toolchain + Cargo (`rust-toolchain.toml`)
- Postgres instance reachable via `DATABASE_URL` (or via Docker Compose `compose.yml`)
- Optional: Docker (to run `compose.yml` services: Postgres + Qdrant)

**Production:**
- Run the `docracy` CLI binary (`crates/cli/src/main.rs`) against a Postgres database (schema managed by `migrations/`)

---

*Stack analysis: 2026-04-05*
