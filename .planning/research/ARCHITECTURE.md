# Architecture Research

**Domain:** Agentic document store / versioned document database for LLM agents (Postgres-backed)
**Researched:** 2026-04-05
**Confidence:** MEDIUM

## Standard Architecture

### System Overview

Agent-facing “document stores” for LLM workflows generally separate **canonical durable state** (documents + revisions) from **derived retrieval views** (search indexes, embeddings, summaries). They also treat “memory” as multiple scopes (thread/session vs long-term), often represented as namespaced documents/collections.

For Docracy (Rust core lib, Postgres first, CLI first), the common structure looks like:

```
┌───────────────────────────────────────────────────────────────────────────┐
│ Interfaces (thin adapters)                                                │
├───────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐   ┌────────────────┐   ┌───────────────────────────┐  │
│  │ CLI (v1)      │   │ Test harness   │   │ Future: API/MCP server     │  │
│  └───────┬───────┘   └───────┬────────┘   └───────────────┬───────────┘  │
│          │                   │                            │              │
├──────────┴───────────────────┴────────────────────────────┴──────────────┤
│ Core library (storage-agnostic)                                           │
├───────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐ │
│  │ Document Domain      │  │ Governance/Seeding   │  │ Query Engine     │ │
│  │ - types/status       │  │ - constitution sync  │  │ - filters/order  │ │
│  │ - revision chaining  │  │ - init context docs  │  │ - pagination     │ │
│  └──────────┬───────────┘  └──────────┬──────────┘  └─────────┬───────┘ │
│             │                          │                       │         │
│             │ emits “revision written” │                       │         │
├─────────────┴──────────────────────────┴───────────────────────┴─────────┤
│ Storage adapters (replaceable) + derived indexes                           │
├───────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────┐      ┌──────────────────────────────┐  │
│  │ Canonical Store (Postgres)    │      │ Derived Retrieval Views       │  │
│  │ - docs (identity + head)      │      │ - full-text index (tsvector)  │  │
│  │ - revisions (append-only)     │      │ - keyword/trigram indexes     │  │
│  │ - blobs/large content (opt)   │      │ - materialized views (opt)    │  │
│  └───────────────────────────────┘      └──────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────┘
```

Key boundary: **canonical “truth”** (docs + immutable revisions) vs **indexes** (rebuildable, may lag, can be recomputed from revisions).

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Interface adapters (CLI/tests/server) | Translate external input/output into core commands/queries; no business rules | CLI args → core API; future HTTP/MCP adapters |
| Command API (Create/Update/Archive/Delete) | Enforce invariants: type/status rules, revision chaining, concurrency checks, audit metadata | “Command handlers” in Rust core; all writes transactional |
| Revision log (append-only) | Store every change as immutable revision records; enable audit + reconstruction | Separate `revisions` table keyed by doc_id + revision_id + parent_revision_id (event-sourcing-like) |
| Document head/materialization | Fast access to current state; pointer to latest revision; derived denormalizations | `documents` table with `head_revision_id`, status, type |
| Governance/constitution sync | Seed and protect immutable governance docs; ensure DB reflects repo-owned constitution | Init routine that upserts seed docs and validates immutability |
| Query engine (filters/search/pagination) | Provide stable query semantics to agents; hide storage details | SQL-backed query builder; hybrid keyword + filters |
| Index maintenance | Keep full-text/search columns up to date from revision content; provide rebuild path | SQL triggers or application-side indexer; idempotent rebuild job |
| Namespacing/tenancy (recommended) | Separate memories by project/user/agent/thread namespaces | Namespace columns / composite keys; aligns with “namespaces + keys” memory patterns |

## Recommended Project Structure

Given “Rust core functions, storage-agnostic; interfaces layered on top”, a Cargo workspace is the cleanest boundary:

```
./
├── crates/
│   ├── docracy-core/              # Domain model + commands/queries
│   │   ├── src/
│   │   │   ├── domain/            # Document types, status, revision model
│   │   │   ├── commands/          # create/read/update/archive semantics
│   │   │   ├── query/             # query DSL + result shaping
│   │   │   ├── governance/        # constitution sync + seed docs
│   │   │   └── storage/           # traits (DocumentRepo, RevisionRepo, QueryRepo)
│   │   └── tests/                 # pure unit tests (no DB)
│   ├── docracy-postgres/          # Postgres adapter implementing storage traits
│   │   ├── src/
│   │   │   ├── repo/              # SQL implementations + row mapping
│   │   │   ├── migrations/        # migration runner (or delegated)
│   │   │   └── search/            # FTS helpers, computed columns
│   │   └── migrations/            # SQL migrations (canonical)
│   └── docracy-cli/               # CLI: init/create/read/query/update
│       └── src/
│           └── main.rs
└── constitution.md                # Repo-owned immutable governance
```

### Structure Rationale

- **docracy-core:** forces clean boundaries (no SQL in domain), improves testability, keeps future backends possible.
- **docracy-postgres:** isolates schema/index choices and migration lifecycle.
- **docracy-cli:** stays thin; helps ensure you can later add HTTP/MCP without refactoring core.

## Architectural Patterns

### Pattern 1: Canonical store + derived indexes (rebuildable)

**What:** Treat the revision store as system-of-record; keep FTS/search artifacts as derived state that can be rebuilt.
**When to use:** Always, if agents rely on search and you anticipate schema/index iteration.
**Trade-offs:** You must build a “reindex/rebuild” path early; read performance improves and schema evolution gets safer.

**Example (conceptual):**

```rust
// Write path: commit revision first; update head + indexes in same transaction.
pub fn update_doc(cmd: UpdateCmd, repo: &dyn Repos) -> Result<DocSnapshot> {
    repo.tx(|tx| {
        let current = tx.docs().get_head(cmd.doc_id)?;
        tx.concurrency().assert_head(cmd.doc_id, cmd.expected_head)?; // CAS/optimistic
        let rev = tx.revisions().append(cmd.doc_id, current.head_rev, cmd.new_content, cmd.meta)?;
        tx.docs().set_head(cmd.doc_id, rev.id, cmd.new_status)?;
        tx.search().upsert_fts(cmd.doc_id, rev.id, rev.render_for_search())?;
        Ok(tx.docs().hydrate(cmd.doc_id)?)
    })
}
```

### Pattern 2: Command/Query separation (CQRS-lite)

**What:** Writes are “commands” that change state and emit immutable revisions; reads are “queries” against materialized current state + search indexes.
**When to use:** When you need stable agent tools: Create/Read/Update/Query with predictable semantics.
**Trade-offs:** Slight duplication (write models vs read models), but it prevents accidental “read modifies state” and keeps index evolution localized.

### Pattern 3: Namespaces as first-class scoping

**What:** Store long-lived knowledge under namespaces (e.g., org/project/agent/thread) instead of a single global bucket.
**When to use:** Multi-agent systems, multi-project environments, or any “memory” feature.
**Trade-offs:** Extra keys in schema and query; huge payoff in preventing cross-contamination and enabling per-scope governance.

## Data Flow

### Request Flow (Write / Update)

```
Agent/CLI
  ↓
Command (Create/Update/Archive)
  ↓ (validate invariants; check governance rules)
Core domain handler
  ↓ (transaction)
Postgres adapter
  ↓
1) append revision (immutable)
2) move document head pointer
3) update derived search artifacts (FTS columns/materialized views)
  ↓
Return: doc snapshot + new revision id
```

### Request Flow (Read / Query)

```
Agent/CLI
  ↓
Query (by id / by filters / by keyword)
  ↓
Query engine
  ↓
Postgres
  ↓
Search/index tables → doc ids → hydrate current snapshots
  ↓
Return: result page (stable ordering + pagination)
```

### Key Data Flows

1. **Init / governance seeding:** repo constitution → governance sync → seed docs inserted/validated → “active context” returned.
2. **Revision chaining:** update command → verify head → append revision with `parent_revision_id` → update head → search refreshed.
3. **Rebuild indexes (operator/dev action):** scan revisions (or current heads) → recompute FTS/search columns → swap/index.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0–1k users / single team | Single Postgres is sufficient; keep everything in one DB; triggers or in-tx updates for FTS |
| 1k–100k users | Add read replicas; consider asynchronous indexer queue if write latency matters; tighten pagination stability |
| 100k+ users | Split retrieval: dedicated search service (e.g., OpenSearch) and/or vector store; treat Postgres as canonical; build CDC/event stream from revisions |

### Scaling Priorities

1. **First bottleneck:** search quality/perf (FTS ranking, trigram, indexes). Fix with better indexing + query plans.
2. **Second bottleneck:** write contention on “head” rows. Fix with optimistic concurrency + short transactions; avoid rewriting large JSON blobs.

## Anti-Patterns

### Anti-Pattern 1: In-place mutation with no immutable history

**What people do:** `UPDATE documents SET content = ...` and call it “versioning” via timestamps.
**Why it’s wrong:** agents need auditability, conflict detection, and reproducible retrieval; you can’t reliably reconstruct state.
**Do this instead:** append immutable revisions and move a head pointer (event-sourcing-like revision log).

### Anti-Pattern 2: Making search indexes “the truth”

**What people do:** store only chunks/embeddings and treat vector/FTS store as canonical.
**Why it’s wrong:** you can’t guarantee lossless reconstruction, governance policies, or deterministic updates.
**Do this instead:** keep a canonical document+revision store; treat embeddings/FTS as derived.

### Anti-Pattern 3: Letting agents edit governance/constitution

**What people do:** store governance as mutable documents without enforcement.
**Why it’s wrong:** the tool contract drifts; agents can “self-authorize” changes.
**Do this instead:** repo-owned immutable constitution synced into DB; changes require code change + review.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Agent frameworks (LangGraph/LangChain, Letta, LlamaIndex, etc.) | Tool-style interface: Init/Create/Read/Query/Update | These ecosystems explicitly distinguish short-term vs long-term memory and often store long-term memory as documents/collections under namespaces |
| Embeddings/vector DB (future) | Optional derived indexer behind a trait | Out of scope for v1 per PROJECT.md; keep boundary so it’s pluggable later |
| Observability (future) | Structured logs + tracing around commands/queries | Especially helpful for debugging agent write behavior |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI ↔ Core | Direct function calls | Keep CLI thin; no SQL |
| Core ↔ Storage traits | Trait interfaces | Enables mocking + future backends |
| Postgres canonical store ↔ derived indexes | Same transaction (v1) or async indexer (later) | Ensure rebuildability; never require indexes to recover documents |
| Governance ↔ Command handlers | Policy checks | Enforce immutability + allowed status transitions |

## Suggested Build Order (dependencies)

1. **Domain model + invariants (core):** document identity, type/status, revision chain, concurrency contract.
2. **Canonical Postgres schema + migrations:** documents + revisions (+ soft delete/archive), minimal indexes.
3. **Core commands (Create/Read/Update/Archive) backed by Postgres adapter:** end-to-end correctness first.
4. **Query engine (filters, ordering, pagination):** stable query semantics for agent tools.
5. **Keyword/full-text search as derived view:** implement as derived columns/indexes; add rebuild path.
6. **Governance seeding + constitution sync:** init path, immutability enforcement, seed docs returned.
7. **CLI + integration tests:** make the tool contract real; lock behavior with tests.
8. **Later:** async indexer/event stream, richer extension querying, optional vector/graph stores.

## Sources

- LangGraph docs: memory overview (short-term vs long-term; stores; namespaces/keys; “hot path vs background” writing) — https://docs.langchain.com/oss/python/concepts/memory (HIGH)
- Letta docs: memory system overview (core blocks always-in-context + archival searchable memory + folders/docs) — https://docs.letta.com/llms.txt (MEDIUM)
- LlamaIndex docs: swappable storage components (docstore/index store/vector store/graph store) — https://docs.llamaindex.ai/en/stable/module_guides/storing/ (MEDIUM)
- PostgreSQL docs: Full Text Search (tsvector, indexes) — https://www.postgresql.org/docs/current/textsearch.html (HIGH)
- PostgreSQL docs: JSON/JSONB types + indexing (for extensions/metadata) — https://www.postgresql.org/docs/current/datatype-json.html (HIGH)
- Martin Fowler: Event Sourcing (revision/event log as source of truth; rebuild state) — https://martinfowler.com/eaaDev/EventSourcing.html (MEDIUM; conceptually foundational but dated)

---
*Architecture research for: agentic document stores / versioned document DB for LLM agents*
*Researched: 2026-04-05*
