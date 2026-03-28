# Project Research Summary

**Project:** Docracy
**Domain:** Postgres-backed, versioned “agentic document store” for LLM agents (Rust core library + CLI first)
**Researched:** 2026-04-05
**Confidence:** MEDIUM

## Executive Summary

Docracy is a durable, queryable “memory bureaucracy” for agentic systems: typed documents with status, append-only revision history, and deterministic query/search primitives. Experts build this class of system by treating **immutable history as the source of truth** (event-sourcing-like revisions) and keeping **retrieval indexes as derived, rebuildable artifacts** (FTS now; vectors later).

The recommended v1 approach is a Rust core library with storage traits and a Postgres adapter, exposing a thin CLI tool surface (Init/Create/Read/Query/Update). Prioritize correctness contracts that agents depend on: stable doc identity vs immutable revision identity, optimistic concurrency (expected-parent/head checks), idempotency for at-least-once tool calls, and a single canonical definition of “latest visible head.”

The main risks are silent history corruption (mutable “revisions”), lost updates from parallel agents, governance drift (agents editing constitution/policy), and search/index drift. Mitigate with DB constraints + transactional command handlers, explicit 409-conflict surfaces, repo-owned constitution hash verification + write-protection, and operational “reindex/backfill” commands from the canonical revision store.

## Key Findings

### Recommended Stack

From [STACK.md](./STACK.md): Use a modern Rust baseline (Edition 2024) with Postgres 17+ and SQL-first access via SQLx. This keeps complex revision/search queries explicit while retaining strong type checks where helpful, and avoids extra infrastructure by relying on Postgres FTS + JSONB + indexing.

**Core technologies:**
- **Rust 1.85.0 (Edition 2024):** core library + CLI — strong invariants and predictable performance for revision/state-machine logic.
- **PostgreSQL 17+:** canonical durable store — transactions, auditability, JSONB, and mature indexing/FTS.
- **SQLx 0.8.6:** async Postgres access + migrations — SQL-first (better for FTS + revision queries) with optional compile-time SQL checks.
- **Tokio 1.50.0:** async runtime — ecosystem default and SQLx-compatible.
- **clap 4.6.0:** CLI parsing — standard Rust CLI UX + derive ergonomics.

Notable recommendations: UUIDv7 for time-sortable IDs; `dotenvy` (not `dotenv`); Postgres FTS with generated `tsvector` + GIN; optional `unaccent` + `pg_trgm` for better UX.

### Expected Features

From [FEATURES.md](./FEATURES.md): v1 must feel like a real versioned store (not “JSON in a table”). Focus on table-stakes that make agent loops safe and deterministic; defer branching/vectors/CRDTs.

**Must have (table stakes):**
- Document CRUD with stable IDs
- Typed docs + required metadata (type, status, timestamps, tags, extensions)
- Immutable revision history + lineage (parent pointer)
- Read latest + read-at-revision
- Optimistic concurrency control (expected parent/head) with explicit conflict errors
- Soft delete/archive semantics
- Query (filters + stable ordering + pagination)
- Keyword search (Postgres FTS) across defined fields
- Provenance/audit metadata (actor/tool/run/reason)
- Namespaces/workspaces (logical separation)
- Migrations + schema versioning

**Should have (competitive, v1.x):**
- Diff + patch primitives (text/JSON) for review and correction
- Export/import (JSONL) for backup/portability
- Conflict inspection + guided resolution tooling
- RBAC / per-workspace permissions (when multi-user)

**Defer (v2+):**
- Time-travel queries (“as-of” commit/time) across collections
- Git-like commits/branching/merge; tamper-evident proofs
- Vector DB mirroring / hybrid retrieval; CRDT/local-first replication

### Architecture Approach

From [ARCHITECTURE.md](./ARCHITECTURE.md): Use a layered design: thin adapters (CLI now, server later) on a storage-agnostic core; Postgres as the first canonical store; derived indexes/search are rebuildable.

**Major components:**
1. **Core domain + command handlers** — enforce invariants (revision chaining, status/type rules, OCC, idempotency, audit metadata).
2. **Storage traits + Postgres adapter** — implement docs/revisions persistence, migrations, and query primitives.
3. **Query/search subsystem** — stable filters/order/pagination + derived FTS/trigram indexes with a rebuild path.
4. **Governance/seeding** — constitution sync, seed docs, immutability enforcement, and status transition policy.
5. **CLI + test harness** — the public tool contract; locks behavior with black-box tests.

Key patterns: canonical store vs derived indexes; CQRS-lite (commands vs queries); namespaces as first-class scoping.

### Critical Pitfalls

From [PITFALLS.md](./PITFALLS.md): the failure mode is “it demos, then agents destroy trust.” Top pitfalls and preventions:

1. **Blurring document vs revision identity** — doc_id stable, revision_id immutable; append-only revisions; head pointer constrained.
2. **No concurrency story (lost updates)** — require expected parent/head; surface 409 conflicts; never silently last-write-wins.
3. **Ordering by timestamps** — define revision ordering by parent pointers / monotonic per-doc sequence; timestamps are metadata.
4. **Schema evolution without version discipline** — include doc_type_version; tolerant parsing + upcasters/backfills.
5. **Governance/constitution drift (or agent mutation)** — repo-owned hash/version checks; write-protect governance docs at API/DB level.
6. **Non-idempotent tools** — idempotency keys + unique constraints; seeding must be re-runnable.
7. **Derived index drift** — search is derived; ship reindex/backfill and drift checks early.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Canonical revision store (correctness-first)
**Rationale:** Every higher-level feature (search, diff, governance, export) depends on immutable revisions + safe writes.
**Delivers:** Postgres schema+migrations for documents + revisions; append-only enforcement; head pointer semantics; OCC; idempotency; soft delete/archive; minimal core API.
**Addresses:** revision history/lineage, read latest/read-at-revision, OCC, migrations, soft delete/archive, provenance scaffolding.
**Avoids:** doc vs revision identity confusion; timestamp-ordering bugs; non-idempotent retries.

### Phase 2: Tool contract + deterministic query primitives
**Rationale:** Agents need stable “Init/Create/Read/Query/Update” semantics before optimizing retrieval.
**Delivers:** Rust core command/query API + Postgres adapter; namespaces/workspaces; filters/order/cursor pagination; CLI commands + integration tests.
**Addresses:** CRUD, typed docs + metadata, namespaces, query semantics, provenance/audit metadata, CLI/test harness.
**Avoids:** “latest-visible” inconsistencies (centralize definition); CLI-as-only-validator (enforce in core+DB).

### Phase 3: Search as derived view + operational rebuild
**Rationale:** Search is the first scaling bottleneck and a major trust surface; do it as rebuildable derived state.
**Delivers:** Postgres FTS (`tsvector`+GIN) on defined fields; optional `unaccent`/`pg_trgm`; `reindex`/backfill command; search/read consistency tests.
**Addresses:** keyword search, predictable retrieval UX.
**Avoids:** treating derived indexes as canonical; search drift without a rebuild path.

### Phase 4: Governance hardening + safe evolution
**Rationale:** Governance is the boundary between “helpful memory” and “agent entropy/attack surface.”
**Delivers:** init seeding with stable IDs + content hash; constitution sync (repo hash/version verification); status/state-machine enforcement; write-protection for governance docs; extension validation policies (size/keys).
**Addresses:** immutable constitution, policy-driven status/type rules, safer extensions.
**Avoids:** governance drift; free-form status chaos; ungoverned JSONB extensions becoming a hidden schema.

### Phase 5: v1.x UX + safety improvements
**Rationale:** After core loop validates, improve human/agent operability without expanding scope to branching/vectors.
**Delivers:** diff/patch views; conflict inspection workflows; export/import (JSONL) with schema versions; RBAC (if multi-user); initial retention/redaction workflow.
**Addresses:** reviewability, portability, safer multi-agent concurrency.
**Avoids:** opaque conflict resolution and irrecoverable secret/PII persistence.

### Phase Ordering Rationale

- Immutable revisions + OCC + idempotency are prerequisites for trustworthy search, diff/patch, and governance.
- Query/pagination correctness must be locked before adding FTS and ranking tweaks.
- Governance is safest once the command/query surface is stable; otherwise “policy” becomes ad-hoc and bypassable.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Governance hardening):** state-machine design per doc type; immutability enforcement strategies (API + DB); extension policy model.
- **Phase 5 (Retention/redaction + RBAC):** security/compliance tradeoffs in append-only history; permission model and least-privilege tool design for agents.

Phases with standard patterns (skip research-phase):
- **Phase 1–3:** Postgres schema/migrations, OCC, and FTS-as-derived-index are well-documented and covered by strong sources.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Versions and capabilities grounded in official/docs.rs sources; clear v1 tradeoffs (Postgres FTS first). |
| Features | MEDIUM | Good competitive landscape, but prioritization depends on target users’ exact agent framework/workflows. |
| Architecture | MEDIUM | Patterns are standard and fit constraints, but operational choices (triggers vs app-side indexing; tenancy model) need validation. |
| Pitfalls | MEDIUM | Strong, actionable risks with credible sources; some mitigations (redaction/crypto-shredding) need deeper design work. |

**Overall confidence:** MEDIUM

### Gaps to Address

- **Concrete doc type system + schemas:** define initial doc types/statuses and their evolution/versioning strategy.
- **Indexing lifecycle choice:** triggers vs in-transaction updates vs async indexer (later) and how to guarantee rebuildability.
- **Security model:** least-privilege tool surface, workspace isolation, and handling untrusted content (“memory poisoning”).
- **Retention/redaction mechanics:** policy + implementation that preserves audit semantics while enabling incident response.

## Sources

### Primary (HIGH confidence)
- PostgreSQL 17 release notes — baseline version choice: https://www.postgresql.org/docs/release/17.0/
- PostgreSQL Full Text Search docs (tsvector/GIN/websearch_to_tsquery): https://www.postgresql.org/docs/17/textsearch-intro.html
- PostgreSQL `pg_trgm` extension docs: https://www.postgresql.org/docs/17/pgtrgm.html
- PostgreSQL `unaccent` extension docs: https://www.postgresql.org/docs/17/unaccent.html
- SQLx docs + repo notes (incl. dotenv vs dotenvy): https://docs.rs/sqlx/latest/sqlx/ and https://github.com/launchbadge/sqlx
- Rust 2024 edition guide / Rust 1.85.0: https://doc.rust-lang.org/edition-guide/rust-2024/
- OWASP AI Agent Security Cheat Sheet (tool abuse, memory poisoning): https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- Microsoft Learn Event Sourcing pattern (ordering/idempotency/schema evolution/reg deletion tension): https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing

### Secondary (MEDIUM confidence)
- Dolt (Git-like SQL DB primitives): https://docs.dolthub.com/introduction/what-is-dolt
- TerminusDB (commits/diff/time-travel concepts): https://github.com/terminusdb/terminusdb
- CouchDB conflicts/revisions (MVCC conflict surfacing): https://docs.couchdb.org/en/stable/replication/conflicts.html
- Martin Fowler — Event Sourcing (conceptual framing): https://martinfowler.com/eaaDev/EventSourcing.html
- LangGraph memory concepts (namespaces/keys; hot-path vs background writes): https://docs.langchain.com/oss/python/concepts/memory

### Tertiary (LOW confidence / needs validation)
- Pinecone ICML 2025 metadata filtering paper (future vector milestone context): https://www.pinecone.io/research/ICML_2025.pdf

---
*Research completed: 2026-04-05*
*Ready for roadmap: yes*
