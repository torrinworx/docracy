# Testing Patterns

**Analysis Date:** 2026-04-05

## Test Framework

**Runner:**
- Rust built-in test harness (`#[test]`, `#[cfg(test)]` modules).
- Async tests use Tokio macros.

**Assertion Library:**
- Standard library assertions (`assert!`, `assert_eq!`, `matches!`).

**Run Commands:**
```bash
cargo test --workspace                              # Run all tests
cargo test -p docracy-core                          # Core crate unit tests
cargo test -p docracy-postgres --test postgres_integration  # Postgres integration test
```

## Test File Organization

**Unit tests (co-located):**
- Inline `#[cfg(test)] mod tests` inside source modules.
  - Examples:
    - `crates/core/src/document.rs`
    - `crates/core/src/service.rs`

**Integration tests (crate-level):**
- Standard Rust integration test layout under a crate’s `tests/` directory.
  - Example: `crates/postgres/tests/postgres_integration.rs`.

**Repository-level `tests/`:**
- Present but empty (`tests/.gitkeep`).

## Test Structure

**Unit test structure pattern:**
- Arrange → Act → Assert in a single test function, using explicit fixture structs.
  - Example fixture setup for determinism:
    - `crates/core/src/service.rs`: `FixedClock` implements `Clock`; `FixedIds`/`SeqIds` implement `IdGenerator`.

**Async unit tests:**
- Use Tokio current-thread runtime for deterministic async behavior:
  - `crates/core/src/service.rs`: `#[tokio::test(flavor = "current_thread")]`.

## Mocking / Test Doubles

**Framework:**
- No mocking framework detected.

**Pattern used instead:**
- Use in-memory repository implementation as the primary test double.
  - `crates/core/src/memory.rs`: `MemoryRepository` implements `Repository`.
  - `crates/core/src/service.rs` tests call service functions with `MemoryRepository`.

## Fixtures and Factories

**Test data construction:**
- Use explicit struct literals + `serde_json::json!` to build content.
  - Examples:
    - `crates/core/src/document.rs`: `NewDocument { ..., content: json!("hi"), ... }`
    - `crates/core/src/service.rs`: `content: json!({"a": 1})`

**Temporary filesystem fixtures:**
- Use `tempfile::TempDir` to write short-lived governance bundles.
  - `crates/core/src/service.rs`: writes `CONSTITUTION.md` for `FsGovernanceSource`.
  - `crates/postgres/tests/postgres_integration.rs`: writes `CONSTITUTION.md` to temp dir.

## Integration Testing (Postgres)

**Database selection:**
- Integration test uses an externally provided Postgres URL.
  - `crates/postgres/tests/postgres_integration.rs`: reads `DOCRACY_TEST_DATABASE_URL` or falls back to `DATABASE_URL`.
  - If neither is set, the test returns early (skips) rather than failing.

**Isolation strategy:**
- Each test run creates a unique schema and sets `search_path` for the connection pool.
  - `crates/postgres/tests/postgres_integration.rs`: creates `docracy_test_<uuid>` schema; uses `PgPoolOptions::after_connect` to `SET search_path`.
- Cleanup is best-effort via a guard `Drop` that drops the schema.

**Migrations:**
- Tests apply real SQL migrations via the adapter.
  - `crates/postgres/src/lib.rs`: `PgRepository::migrate()` runs `sqlx::migrate!("../../migrations")`.
  - Migration files live at repository root: `migrations/0001_documents_and_revisions.sql`, `migrations/0002_single_constitution.sql`, `migrations/0003_content_search.sql`.

## Coverage

**Requirements:**
- None enforced (no coverage tooling/config detected).

## Test Types

**Unit tests:**
- Focus on validation rules, deterministic ID/time behavior, and core service flows.
  - Examples:
    - `crates/core/src/document.rs`: slug validation; timestamp/status invariants.
    - `crates/core/src/service.rs`: revision chain correctness; init bootstrapping/repair; query default behavior.

**Integration tests:**
- Exercise real Postgres adapter behavior, migrations, and end-to-end service flows against the database.
  - Example: `crates/postgres/tests/postgres_integration.rs`.

**E2E tests:**
- Not detected (no CLI black-box tests, no HTTP server tests).

## Common Patterns

**Error testing:**
- Use `unwrap_err()` for validation failures and `matches!` for enum error variants.
  - `crates/core/src/document.rs`: `nd.validate().unwrap_err()`.
  - `crates/postgres/tests/postgres_integration.rs`: `assert!(matches!(res, Err(docracy_core::RepoError::Conflict)));`.

**Async testing:**
- Prefer `#[tokio::test]` and await service calls directly.
  - `crates/core/src/service.rs`, `crates/postgres/tests/postgres_integration.rs`.

---

*Testing analysis: 2026-04-05*
