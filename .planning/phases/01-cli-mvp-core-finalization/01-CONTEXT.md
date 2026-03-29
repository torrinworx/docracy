# Phase 01: CLI MVP + Core Finalization - Context

**Gathered:** 2026-04-05
**Status:** Ready for execution
**Source:** User directive + roadmap re-scope

<domain>
## Phase Boundary

This phase finalizes the CLI-backed MVP: stable document/revision writes, governance init, deterministic query/search, and clean JSON CLI behavior.

</domain>

<decisions>
## Implementation Decisions

### Core document lifecycle
- Updates must require an expected head revision and reject stale writes.
- Revision history remains immutable; updates append a new revision and advance the head atomically.

### Governance
- `constitution.md` is repo-owned and immutable to supported user interfaces.
- `init` may reconcile the database to the repo-owned constitution as a system action.

### CLI contract
- Commands continue to use JSON I/O.
- Failures must be machine-readable and non-zero exit.

### the agent's Discretion
- Exact internal helper names, error wording, and output envelope shape as long as the contracts above hold.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — validated scope, active requirements, and constraints
- `.planning/ROADMAP.md` — phase goals and requirement mapping
- `.planning/STATE.md` — current phase position and execution state

### Core implementation
- `crates/core/src/service.rs` — create/read/update/init orchestration
- `crates/core/src/query.rs` — query parsing and projection
- `crates/core/src/document.rs` — document lifecycle and reserved constitution type
- `crates/core/src/revision.rs` — revision shape and validation
- `crates/core/src/errors.rs` — core and repository error types
- `crates/core/src/repository.rs` — storage boundary contract
- `crates/core/src/memory.rs` — in-memory repository behavior
- `crates/core/src/governance.rs` — governance bundle loading

### Postgres and CLI
- `crates/postgres/src/lib.rs` — Postgres adapter and transactional behavior
- `crates/cli/src/main.rs` — CLI surface and JSON contract
- `migrations/0001_documents_and_revisions.sql` — base schema and invariants
- `migrations/0002_single_constitution.sql` — constitution uniqueness
- `migrations/0003_content_search.sql` — content search index

### Governance source
- `governance/CONSTITUTION.md` — repo-owned constitution text

</canonical_refs>

<specifics>
## Specific Ideas

- `update` should accept the expected head revision in JSON input.
- Query output should stay stable for filtering, pagination, and unsupported `extensions` access.
- `init` should stay rerunnable and return active `context` documents.

</specifics>

<deferred>
## Deferred Ideas

- Direct-core test harness remains Phase 2.
- Any broader interface work (server/MCP) stays out of this phase.

</deferred>

---

*Phase: 01-cli-mvp-core-finalization*
*Context gathered: 2026-04-05 via user directive*
