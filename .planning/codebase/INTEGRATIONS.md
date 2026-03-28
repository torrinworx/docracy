# External Integrations

**Analysis Date:** 2026-04-05

## APIs & External Services

**Databases / Storage Services:**
- PostgreSQL - primary durable store for documents + revisions + search
  - Client: SQLx `0.8.6` (`crates/postgres/src/lib.rs`)
  - Migrations: `migrations/0001_documents_and_revisions.sql`, `migrations/0002_single_constitution.sql`, `migrations/0003_content_search.sql`
  - Connection: `DATABASE_URL` (CLI uses `--database-url` override) (`crates/cli/src/main.rs`)
  - Notable DB features used:
    - `pgcrypto` extension for `gen_random_uuid()` (`migrations/0001_documents_and_revisions.sql`)
    - Full-text search generated `tsvector` + GIN index (`migrations/0003_content_search.sql`)
    - JSONB `extensions` with GIN index (`migrations/0001_documents_and_revisions.sql`)

- Qdrant (vector database) - provisioned for local runs only; not currently called from Rust code
  - Provisioning: `compose.yml` service `qdrant`
  - Storage: `./qdrant_data/` mounted into container (`compose.yml`)

## Data Storage

**Databases:**
- PostgreSQL (container image `postgres:16` for local dev via `compose.yml`)
  - Connection string: `DATABASE_URL` (app/tests) (`crates/cli/src/main.rs`, `crates/postgres/tests/postgres_integration.rs`)

**File Storage:**
- Local filesystem only
  - Docker volumes (dev): `./postgres_data/`, `./qdrant_data/` (`compose.yml`)
  - Governance markdown loaded from filesystem: default `./governance/` (`crates/cli/src/main.rs`, `crates/core/src/governance.rs`)

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- Not applicable (CLI-only; no server endpoints detected)

## Monitoring & Observability

**Error Tracking:**
- None detected

**Logs:**
- CLI prints JSON to stdout/stderr; errors serialized as JSON (`crates/cli/src/main.rs`)

## CI/CD & Deployment

**Hosting:**
- Not detected (no deployment configuration in repo; `.github/` not present)

**CI Pipeline:**
- Not detected

## Environment Configuration

**Required env vars:**
- `DATABASE_URL` - Postgres connection string for CLI/runtime (`crates/cli/src/main.rs`)

**Optional env vars:**
- `DOCRACY_TEST_DATABASE_URL` - Postgres connection string for integration tests (`crates/postgres/tests/postgres_integration.rs`)

**Docker Compose env vars (local dev):**
- Postgres service (`compose.yml`): `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_PORT`
- Qdrant service (`compose.yml`): `QDRANT_HTTP_PORT`, `QDRANT_GRPC_PORT`

**Secrets location:**
- `.env` present (environment configuration; do not read/commit secrets)
- `.env.example` present (environment template; do not treat as secret, but do not embed values in committed docs)

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- None detected

---

*Integration audit: 2026-04-05*
