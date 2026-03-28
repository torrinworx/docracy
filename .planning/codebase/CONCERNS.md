# Codebase Concerns

**Analysis Date:** 2026-04-05

## Tech Debt

**Docs vs implementation drift (query + read semantics):**
- Issue: README describes richer `Query.where` semantics (notably `extensions.*` filters and `.contains`) and `Read.include`, but the implementation rejects extension filters and `Read` only accepts `ids`.
- Files:
  - `README.md` (example `extensions.*` where filters; `Read.include`)
  - `crates/core/src/query.rs` (rejects `where` keys starting with `extensions.`; only supports a fixed allowlist)
  - `crates/cli/src/main.rs` (`ReadInput { ids: Vec<DocumentId> }`)
- Impact: callers following README will hit validation errors or missing functionality; hard to build clients that match expected API.
- Fix approach:
  - Update README examples to match current `QueryInput::parse` behavior, or implement the documented extension filtering (incrementally, starting with exact-match keys on `documents.extensions`).
  - Align `Read` CLI/input structs with documented fields or remove the docs.

**Dual source of truth for content (document vs revision):**
- Issue: `documents.content` and `document_revisions.content` both store content; correctness depends on always updating both consistently.
- Files:
  - `migrations/0001_documents_and_revisions.sql` (`documents.content`, `document_revisions.content`)
  - `crates/postgres/src/lib.rs` (`update_document_with_revisions` updates both; `update_document` updates only `documents`)
  - `crates/core/src/service.rs` (service path uses `update_document_with_revisions`, but the trait exposes `update_document` directly)
- Impact: adapters/callers using `Repository::update_document` (or direct SQL) can desync the “current document” content from the latest revision history.
- Fix approach:
  - Restrict/privatize `Repository::update_document` if it’s not meant for normal writes, or document strict invariants.
  - Add a database-side invariant (e.g., trigger or deferred constraint) ensuring `documents.current_revision_id` points to a revision whose `content/extensions` match the document row.

**Cursor + ORDER BY mismatch with existing indexes:**
- Issue: keyset pagination compares tuples `(modified_at, id)` / `(created_at, id)` but schema only indexes `modified_at` and not the composite.
- Files:
  - `crates/postgres/src/lib.rs` (`query_documents` cursor filters on `(modified_at, id)` / `(created_at, id)`)
  - `migrations/0001_documents_and_revisions.sql` (indexes on `documents_modified_at_idx`; no composite `(modified_at, id)` / `(created_at, id)`)
- Impact: queries can degrade into sorts + scans under load, especially with cursor pagination and status/type filters.
- Fix approach:
  - Add composite indexes aligned to query patterns, e.g. `(modified_at, id)` and `(created_at, id)`; consider partial indexes for common filters (e.g., `status='active' AND deleted_at IS NULL`).

**Rust edition/tooling baseline mismatch with intended direction:**
- Issue: workspace uses Rust 2021 editions and UUIDv4 IDs.
- Files:
  - `crates/core/Cargo.toml`, `crates/postgres/Cargo.toml`, `crates/cli/Cargo.toml` (edition = "2021")
  - `crates/core/src/service.rs` (`UuidV4Generator`)
- Impact: harder to adopt newer edition features and time-sortable IDs (useful for locality and pagination).
- Fix approach:
  - If desired, bump edition to 2024 in a single chore PR; switch IDs to UUIDv7 (or ULID) behind a feature flag to avoid breaking existing DBs.

## Known Bugs

**`docracy migrate --no-migrate` reports success without migrating:**
- Symptoms: `Migrate` subcommand outputs `{ "ok": true }` regardless of whether migrations ran.
- Files: `crates/cli/src/main.rs` (migrations are only run via the global `if !cli.no_migrate { repo.migrate()... }`; `Command::Migrate` does not call `repo.migrate()`)
- Trigger: run `docracy --no-migrate migrate`.
- Workaround: omit `--no-migrate` for the migrate command.

## Security Considerations

**No authentication/authorization boundary:**
- Risk: anyone who can run the CLI (or future server) with DB credentials can read/write all documents, including governance and context documents.
- Files:
  - `crates/cli/src/main.rs` (direct DB access; no auth)
  - `crates/core/src/service.rs` (core operations accept any caller)
- Current mitigation: relies on Postgres credentials + host/network isolation.
- Recommendations:
  - Treat DB role as the primary security boundary (least-privilege user; separate read-only role where possible).
  - For any network-exposed interface, add explicit authn/z and per-document access policy before exposing `Create/Read/Query/Update`.

**Potential sensitive data leakage via raw error strings:**
- Risk: CLI prints `err.to_string()` to stderr; backend/storage errors can include detailed DB messages.
- Files:
  - `crates/cli/src/main.rs` (top-level error JSON uses `err.to_string()`)
  - `crates/postgres/src/lib.rs` (`map_sqlx_error` formats DB errors with `format!("{db}")`)
- Current mitigation: none.
- Recommendations:
  - Redact connection strings and avoid printing raw DB errors by default; include a `--verbose` flag to emit full details.
  - Normalize storage errors into stable, user-safe error codes/messages.

**Governance content is returned verbatim by Init:**
- Risk: `init` returns the full content of every `.md` file in the governance directory; this can leak policy documents or other sensitive governance material if the directory contains it.
- Files:
  - `crates/core/src/governance.rs` (loads all `*.md` in a directory)
  - `crates/cli/src/main.rs` (serializes governance files including `content`)
- Current mitigation: governance directory content is assumed safe.
- Recommendations:
  - Consider returning governance file hashes/metadata by default and requiring an explicit flag to include full contents.
  - Enforce an allowlist (`CONSTITUTION.md`, etc.) rather than reading all markdown files.

**Secrets and local state present in repo root (must remain uncommitted):**
- Risk: accidental commit of local env config or DB data directories.
- Files:
  - `.env`, `.env.example` (present; treated as secrets/config)
  - `postgres_data/`, `qdrant_data/` (local state)
  - `.gitignore` (ignores env files + local data dirs)
- Current mitigation: `.gitignore` entries exist.
- Recommendations:
  - Add a pre-commit/CI check to fail if `.env*` or `*_data/` directories are staged.

**Input validation gaps for resource abuse (DoS-style):**
- Risk: very large `content` / `extensions` payloads or extremely large `cursor` strings can cause memory/CPU spikes.
- Files:
  - `crates/core/src/document.rs` (no size limits on `content`)
  - `crates/core/src/query.rs` (`decode_cursor` base64-decodes without max length checks)
  - `crates/cli/src/main.rs` (reads stdin into a full `String`)
- Current mitigation: none.
- Recommendations:
  - Enforce size limits at interface boundaries (CLI/server), and optionally in core validation.
  - Add cursor length checks before base64 decode.

## Performance Bottlenecks

**`query_documents` always does a `COUNT(*)` over the full match set:**
- Problem: pagination performs a separate count query that ignores cursors, which can be expensive on large datasets.
- Files: `crates/postgres/src/lib.rs` (`query_documents`: `SELECT COUNT(*) ...` built via `QueryBuilder`)
- Cause: accurate total count computed per query.
- Improvement path:
  - Make total count optional (e.g., caller flag), or return an estimate for large datasets.
  - Ensure indexes cover the common filter combinations to keep count fast.

**Full-text search over entire JSON content:**
- Problem: `content_search_tsv` is generated from `content::text`, which can be large/noisy for deeply nested JSON.
- Files: `migrations/0003_content_search.sql` (`to_tsvector('english', coalesce(content::text, ''))`)
- Cause: indexing the whole JSON serialization.
- Improvement path:
  - Consider extracting searchable text fields into a dedicated column or a computed text projection.
  - Add guardrails on content size or depth; consider language configuration and optional `unaccent`.

**Extensions GIN index may be oversized and underutilized:**
- Problem: GIN index on all of `extensions` can grow quickly; current query layer explicitly disallows `extensions.*` filtering.
- Files:
  - `migrations/0001_documents_and_revisions.sql` (GIN on `documents.extensions` and `document_revisions.extensions`)
  - `crates/core/src/query.rs` (rejects `where` keys starting with `extensions.`)
- Cause: schema anticipates extension queries but API layer blocks them.
- Improvement path:
  - Either implement extension filtering, or drop/replace the broad GIN index with targeted indexes once extension query shapes are known.

## Fragile Areas

**Data integrity not fully enforced for revision/document relationships:**
- Files:
  - `migrations/0001_documents_and_revisions.sql` (FK `documents.current_revision_id -> document_revisions.id`, but no constraint tying that revision to the same `document_id`)
  - `migrations/0001_documents_and_revisions.sql` (`parent_revision_id` FK exists, but doesn’t enforce same `document_id` as child)
- Why fragile: a buggy adapter or manual SQL can create cross-document revision chains or point a document at another document’s revision; the DB will accept it.
- Safe modification:
  - Add constraints/triggers that enforce: `document_revisions.parent_revision_id` (if set) must refer to a revision with the same `document_id`; and `documents.current_revision_id` must refer to a revision with matching `document_id`.
  - Add a consistency check query in CI.
- Test coverage: partial (integration test covers “single constitution” uniqueness and basic flows, not corruption scenarios).

**Governance file loading has no allowlist/size limits:**
- Files: `crates/core/src/governance.rs`, `crates/cli/src/main.rs`
- Why fragile: unexpected large/extra markdown files can bloat responses and slow init.
- Safe modification: add allowlist + max file size checks.
- Test coverage: core service tests cover bootstrap/repair behavior, not governance bundle bounds.

## Scaling Limits

**Document size and index growth:**
- Current capacity: not bounded in code.
- Limit: large JSON `content` + generated tsvector and GIN indexes can grow quickly; query/count costs rise with dataset size.
- Scaling path:
  - Define size limits per document type.
  - Introduce archiving/compaction policies and consider partitioning strategies if needed.

## Dependencies at Risk

**Chrono + UUIDv4 baseline:**
- Risk: not inherently unsafe, but UUIDv4 hurts index locality; chrono is heavier than newer time crates.
- Impact: performance/maintenance friction rather than correctness.
- Migration plan: evaluate UUIDv7 and time handling once DB/compatibility strategy is defined.

## Missing Critical Features

**Extension querying is intentionally blocked despite indexing support:**
- Problem: schema includes `extensions` JSONB + GIN indexes, but `QueryInput::parse` rejects `where` keys starting with `extensions.`.
- Blocks: efficient retrieval by agent-defined metadata.
- Files: `crates/core/src/query.rs`, `migrations/0001_documents_and_revisions.sql`

**Backups/restore and operational runbook:**
- Problem: repository includes `compose.yml` and local DB data dirs, but lacks documented backup/restore procedures.
- Blocks: safe adoption in production-like environments.
- Files: `compose.yml` (present), `README.md` (no backup guidance)

## Test Coverage Gaps

**Repository invariants and corruption scenarios:**
- What's not tested: constraints that `current_revision_id` belongs to the same document; parent revision document_id matching; behavior under concurrent updates.
- Files:
  - `migrations/0001_documents_and_revisions.sql`
  - `crates/postgres/src/lib.rs`
  - `crates/postgres/tests/postgres_integration.rs`
- Risk: subtle data corruption can accumulate without detection.
- Priority: High

**CLI behavior around migrate/init output and error redaction:**
- What's not tested: `--no-migrate` interaction with `migrate` subcommand; output size handling; redaction behavior.
- Files: `crates/cli/src/main.rs`
- Risk: operational surprises; accidental leakage of verbose internal errors.
- Priority: Medium

---

*Concerns audit: 2026-04-05*
