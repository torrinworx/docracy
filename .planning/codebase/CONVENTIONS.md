# Coding Conventions

**Analysis Date:** 2026-04-05

## Naming Patterns

**Crates / packages:**
- Kebab-case crate names in `Cargo.toml` (e.g., `crates/core/Cargo.toml`: `docracy-core`, `crates/postgres/Cargo.toml`: `docracy-postgres`, `crates/cli/Cargo.toml`: `docracy-cli`).

**Files / modules:**
- Rust source files use `snake_case.rs` (e.g., `crates/core/src/document.rs`, `crates/core/src/service.rs`).
- Public module declarations in `crates/core/src/lib.rs` mirror filenames (`pub mod document;`, `pub mod service;`, etc.).

**Types:**
- Structs/enums use `PascalCase` (e.g., `crates/core/src/document.rs`: `Document`, `DocumentType`, `DocumentStatus`).
- Newtype wrappers for IDs/types are common (e.g., `crates/core/src/ids.rs`: `pub struct DocumentId(pub Uuid);`).

**Functions / methods:**
- Functions and methods use `snake_case` (e.g., `crates/core/src/service.rs`: `create_document`, `init_bundle`, `update_document`).
- Constructors are consistently named `new(...)` and return a validation result (e.g., `crates/core/src/document.rs`: `DocumentType::new(...) -> ValidationResult<Self>`).

**Constants:**
- String constants are `SCREAMING_SNAKE_CASE` (e.g., `crates/core/src/document.rs`: `DocumentType::CONSTITUTION`, `DocumentStatus::ARCHIVED`).

## Code Style

**Formatting:**
- Rustfmt is the canonical formatter (no `rustfmt.toml` detected; defaults apply).
  - Toolchain declares rustfmt component in `rust-toolchain.toml`.
- Indentation is 4 spaces; line wrapping appears rustfmt-driven (see `crates/postgres/src/lib.rs` for long SQL string formatting and method chaining).

**Linting:**
- Unsafe is forbidden at the crate level:
  - `crates/core/src/lib.rs`: `#![forbid(unsafe_code)]`
  - `crates/postgres/src/lib.rs`: `#![forbid(unsafe_code)]`
  - `crates/cli/src/main.rs`: `#![forbid(unsafe_code)]`
- Clippy is available in the toolchain (`rust-toolchain.toml` includes `clippy`), but no repository-level clippy configuration files detected (no `clippy.toml`, no lint sections in Cargo manifests).

## Import Organization

**Pattern:**
- Imports are grouped by origin (crate/local first, then third-party, then std), but ordering varies slightly per file.
  - Example (crate + third-party + std): `crates/cli/src/main.rs` imports `docracy_core::*`, then `serde*`, then `std::*`.
  - Example (crate + third-party): `crates/core/src/service.rs` imports `crate::*`, then `chrono`, `serde_json`, `uuid`.

## Error Handling

**Core crate (`docracy-core`)**
- Domain errors are explicit enums using `thiserror`:
  - `crates/core/src/errors.rs`: `RepoError`, `GovernanceError`, `CoreError`.
- Service-layer functions return `Result<..., CoreError>` and convert lower-level errors via `#[from]` where appropriate:
  - `crates/core/src/errors.rs`: `CoreError::Validation(#[from] ValidationError)`, `CoreError::Repo(#[from] RepoError)`, etc.
- Validation is a first-class pattern:
  - `crates/core/src/validation.rs`: `pub type ValidationResult<T> = Result<T, ValidationError>`.
  - Entities expose `validate()` and constructors validate early (e.g., `crates/core/src/document.rs`, `crates/core/src/revision.rs`).

**Postgres adapter (`docracy-postgres`)**
- Storage errors are mapped into `RepoError` at the adapter boundary:
  - `crates/postgres/src/lib.rs`: `fn map_sqlx_error(e: sqlx::Error) -> RepoError` maps unique violations (`23505`) to `RepoError::Conflict`, everything else to `RepoError::Storage(...)`.
- JSON shape validation is treated as storage error:
  - `crates/postgres/src/lib.rs`: `value_to_object_map` enforces `extensions` is an object.

**CLI (`docracy-cli`)**
- CLI boundary uses `anyhow::Result` plus context strings for ergonomics:
  - `crates/cli/src/main.rs`: `.context("failed to connect to postgres")?`, `.context("invalid JSON")`.
- Errors are rendered as JSON on stderr and exit code is non-zero:
  - `crates/cli/src/main.rs`: `main()` prints `{ "error": "..." }` and `std::process::exit(1)`.

## Logging / Output

- No structured logging framework is used in code (no `tracing`/`log` usage detected).
- Output is primarily JSON over stdout/stderr:
  - `crates/cli/src/main.rs`: `println!("{s}")` for successful output, `eprintln!(...)` for error JSON.

## Comments & Docs

- Public traits and key boundaries use doc comments (`///`):
  - `crates/core/src/repository.rs`: documents the repository boundary and adapter intent.
- Inline comments are used to encode invariants and phase constraints:
  - `crates/core/src/document.rs`: “For phase 1, keep status/timestamp rules simple and strict.”
  - `crates/core/src/query.rs`: default filtering behavior rationale.

## Function Design

- Prefer explicit input/output structs for service functions:
  - `crates/core/src/service.rs`: `CreateDocumentResult`, `ReadDocumentsResult`, `UpdateDocumentInput`, `UpdateDocumentResult`, `InitBundleResult`.
- Favor early returns for invalid inputs:
  - `crates/core/src/service.rs`: `if input.content.is_none() && ... { return Err(CoreError::NoChanges); }`.
- Async boundaries are explicit and centralized in service/repository layers.

## Module Design

- Core crate acts as a façade by re-exporting common types/functions from `crates/core/src/lib.rs`.
- Repository is defined as an `async_trait` to keep core storage-agnostic:
  - `crates/core/src/repository.rs`.

## CI Checks, Workflows, and Quality Gates

**CI workflows:**
- Not detected (no `.github/workflows/*`, no `.gitlab-ci.yml`, no `.circleci/`).

**Local quality gates (what to run):**
- Format: `cargo fmt --all` (tooling present via `rust-toolchain.toml`).
- Lint: `cargo clippy --all-targets --all-features` (tooling present via `rust-toolchain.toml`).
- Test: `cargo test --workspace`.

**Environment files:**
- `.env` and `.env.example` are present (repo ignores `.env*` via `.gitignore`); do not commit secrets.

---

*Convention analysis: 2026-04-05*
