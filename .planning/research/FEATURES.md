# Feature Research

**Domain:** Agentic document store / versioned document database for LLM agents
**Researched:** 2026-04-05
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Document CRUD (Create/Read/Update) with stable IDs** | Any “doc store” must reliably persist and fetch docs | MEDIUM | Include idempotent create, predictable IDs, and clear error modes |
| **Typed documents + required metadata** | Agents need structured retrieval (by type/status/updatedAt) | MEDIUM | Type, status, title, timestamps, tags, and arbitrary metadata/`extensions` object |
| **Revision history (immutable revisions) + lineage** | “Versioned DB” implies auditability and safe evolution | HIGH | Store revisions as append-only records; maintain parent revision pointer/chain |
| **Read latest + read-at-revision** | Agents need reproducibility and rollback/inspection | MEDIUM | Retrieve the current “head” and any historical revision by id/revision |
| **Optimistic concurrency control (compare-and-swap on parent rev)** | Multi-agent edits otherwise silently stomp each other | MEDIUM | Require caller to provide expected parent revision; reject on mismatch (conflict) |
| **Soft delete / archive semantics** | Agents need reversibility + retention without data loss | LOW | Prefer `archived_at`/`deleted_at` flags; keep revisions discoverable |
| **Query: filters + ordering + pagination** | Agent loops need deterministic iteration over docs | MEDIUM | Filter by type/status/tags/time; order stable; cursor pagination preferred |
| **Keyword search (full-text) across doc content + key fields** | Users expect global search beyond browsing | HIGH | Postgres FTS works well; define searchable fields and language config |
| **Provenance / audit metadata** | Agents must justify “why did this change?” | MEDIUM | Store actor (agent/human), tool/run id, message/intent, timestamps |
| **Import/export (backup & restore)** | Moving between environments and debugging requires portability | MEDIUM | JSONL export of docs + revisions; restore preserves ids/revision graph |
| **Namespaces / projects (logical separation)** | Teams/agents need multiple isolated workspaces | MEDIUM | Simple: `workspace_id` on every doc + query scoping |
| **Migrations + schema versioning** | Postgres-backed system must evolve safely | MEDIUM | Versioned migrations; forward-only migrations for v1 |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Git-like commits for doc mutations (commit message + author) + “log”** | Makes changes reviewable/debuggable for humans and agents | MEDIUM | Model “commit” as a batch of doc revision writes with metadata |
| **Time-travel queries (“as-of” a commit/time) for collections** | Reproducible agent runs; compare “before vs after” project context | HIGH | Requires stable snapshot semantics; can be implemented via revision tables + commit pointers |
| **Diff & patch primitives (markdown/text diff + JSON patch)** | Enables lightweight review, agent self-correction, and merges | HIGH | “Diff between revisions” and “apply patch as new revision” APIs |
| **Three-way merge support (base/head/other)** | Resolves parallel agent edits without manual rewrite | HIGH | Start with text/JSON merge; keep deterministic rules; surface conflicts |
| **Branching / forks for what-if planning** | Agents can explore alternatives without corrupting canonical context | HIGH | Similar to Dolt/TerminusDB concepts; likely v2+ |
| **Policy-driven governance layer (status transitions, immutables, required fields)** | Prevents agents from “rewriting rules” and reduces entropy | HIGH | Enforce constitution immutability; transition rules per doc type/status |
| **Tamper-evident history proofs** | Higher trust for audit trails and regulated environments | HIGH | Inspired by immudb’s cryptographic verification; optional for v2 |
| **Conflict introspection tools (show conflicting revisions, winner selection, resolution workflows)** | Makes multi-agent concurrency manageable | MEDIUM | Borrow CouchDB’s “conflicts as branches” concept even if storage differs |
| **Automatic compaction/snapshots (summaries) with explicit provenance** | Keeps context usable as history grows | HIGH | Produce derived “snapshot” docs; never overwrite raw history |
| **Agent-oriented retrieval helpers (token-budget aware, “get current context pack”)** | Faster and cheaper agent runs; fewer hallucinations | MEDIUM | Return curated bundles; still backed by deterministic query rules |
| **Extension-aware indexing (governed JSONB indexes)** | Enables power users without schema churn | HIGH | Guardrails: only allow extension queries/indexing when governance permits |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Built-in vector DB / embeddings pipeline (v1)** | “RAG needs vectors” | Adds infra + model lifecycle complexity; conflicts with v1 scope | Keep keyword/filters first; add vector mirroring later (explicitly v2) |
| **Real-time collaborative rich-text editor (CRDT) in v1** | “Notion/Docs-style editing” | Huge surface area (presence, cursors, storage format, sync) | Start with doc revisions + OCC; consider CRDT later (Automerge/Yjs patterns) |
| **Automatic LLM rewriting of historical revisions** | “Keep things tidy” | Destroys auditability and reproducibility; introduces nondeterminism | Use append-only revisions + derived summary docs with provenance |
| **Opaque auto-merge of conflicts via LLM** | “Resolve conflicts automatically” | Non-deterministic + hard to trust; can silently lose facts | Deterministic merge rules; surface conflicts; optionally suggest merges as drafts |
| **Full workflow engine / task tracker baked into the DB** | “Track everything here” | Becomes a generalized product-management suite | Keep doc store primitive + extensible; build workflows as higher-level docs/tools |

## Feature Dependencies

```
[Document CRUD]
    └──requires──> [Typed documents + metadata]
                     └──requires──> [Namespaces / projects]

[Revision history + lineage]
    └──requires──> [Read-at-revision]
                     └──enables──> [Diff & patch]

[Optimistic concurrency control]
    └──enables──> [Conflict introspection tools]
                     └──enables──> [Three-way merge]

[Migrations + schema versioning]
    └──requires──> [Postgres-backed storage]

[Keyword search]
    └──requires──> [Stable content fields + indexing strategy]

[Policy-driven governance layer]
    └──requires──> [Typed documents + status]
                     └──requires──> [Immutable constitution seed]

[Branching / forks]
    └──requires──> [Commit model + time-travel queries]
```

### Dependency Notes

- **Revision history enables diff/patch:** without immutable revisions, you can’t reliably compute diffs or apply patches as new states.
- **OCC enables safe multi-agent writes:** rejecting stale writes forces callers to rebase/merge instead of overwriting.
- **Governance depends on types/status:** policies are keyed by doc type and state; enforcing them without these primitives becomes ad-hoc.
- **Branching depends on a commit model:** “forking” needs a stable notion of a database state pointer (commit/revision set).

## MVP Definition

### Launch With (v1)

- [ ] **Postgres-backed versioned documents** (type + status + revision chain) — core value (durable context with history)
- [ ] **Init seeding of governance docs (including immutable constitution)** — teaches agents how to use the system; prevents drift
- [ ] **Create/Read/Update** with **expected-parent revision** checks — safe multi-agent evolution
- [ ] **Soft delete/archive** — reversible lifecycle management
- [ ] **Query** (filters + ordering + pagination) — deterministic agent loops
- [ ] **Keyword search** (FTS) — global retrieval without vectors
- [ ] **CLI + test harness** (unit + integration) — validates correctness and supports automation

### Add After Validation (v1.x)

- [ ] **Diff views + patch application** — improves reviewability and correction workflows
- [ ] **Export/import** (JSONL + schema version) — portability and backups
- [ ] **Conflict inspection + guided resolution** — better multi-agent concurrency UX
- [ ] **RBAC / per-workspace permissions** — necessary for multi-user environments
- [ ] **Attachments (blob pointers + checksums)** — large artifacts without bloating DB rows

### Future Consideration (v2+)

- [ ] **Time-travel queries across collections (“as-of” commit/time)** — reproducibility at scale
- [ ] **Branching/forking + merge** — experiment safely; PR-style review for docs
- [ ] **Tamper-evident proofs** — stronger audit guarantees (cryptographic verification)
- [ ] **CRDT/local-first replication** — offline-first and real-time collaboration
- [ ] **Vector DB mirroring / hybrid search** — only after core doc store is stable

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Versioned docs (revisions + lineage) | HIGH | HIGH | P1 |
| CRUD + typed docs + status | HIGH | MEDIUM | P1 |
| Query (filters/order/pagination) | HIGH | MEDIUM | P1 |
| Keyword search (FTS) | HIGH | HIGH | P1 |
| Init seeding + immutable constitution | HIGH | MEDIUM | P1 |
| Soft delete/archive | MEDIUM | LOW | P1 |
| Provenance/audit metadata | HIGH | MEDIUM | P1 |
| Diff + patch | MEDIUM | HIGH | P2 |
| Export/import | MEDIUM | MEDIUM | P2 |
| Conflict inspection + resolution tooling | MEDIUM | MEDIUM | P2 |
| RBAC / permissions | MEDIUM | HIGH | P2 |
| Time-travel queries (as-of) | MEDIUM | HIGH | P3 |
| Branching/merge | MEDIUM | HIGH | P3 |
| Tamper-evident proofs | LOW-MEDIUM | HIGH | P3 |
| CRDT replication | LOW-MEDIUM | VERY HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Competitor A | Competitor B | Our Approach |
|---------|--------------|--------------|--------------|
| Git-like version control primitives | **Dolt**: fork/clone/branch/merge for SQL DB | **TerminusDB**: commits, diff, push/pull/clone | Start with revision chains per doc; consider commit objects + branching later |
| Conflict model / concurrency | **CouchDB**: MVCC `_rev`, conflicts as revision tree | **Yjs/Automerge**: CRDT merge without conflicts | v1: OCC + explicit conflict errors; v2: optional conflict inspection/merge tools |
| Time travel / history queries | **Datomic**: immutable datoms, “as-of” filtering | **TerminusDB**: query any state at any commit | Roadmap: add “as-of commit” queries after core revision model is stable |
| Tamper-evident audit history | **immudb**: cryptographic verification, immutable history | (—) | Optional v2+ mode for high-trust audit trails |
| Agent memory / context assembly | **Zep**: “context engineering”, agent memory, Graph RAG | **LlamaIndex**: docstores for nodes, persistence backends | Keep storage primitives deterministic; add higher-level “context pack” helpers later |

## Sources

- Dolt: “a SQL database you can fork/clone/branch/merge like Git” — https://docs.dolthub.com/introduction/what-is-dolt
- TerminusDB README (commits, diff, push/pull/clone, time-travel queries) — https://github.com/terminusdb/terminusdb
- CouchDB docs on revisions/conflicts (MVCC `_rev`, conflict handling) — https://docs.couchdb.org/en/stable/replication/conflicts.html
- PostgreSQL Full Text Search docs (FTS primitives) — https://www.postgresql.org/docs/current/textsearch.html
- immudb README (immutable DB, cryptographic proof/verification) — https://github.com/codenotary/immudb
- Automerge README (CRDTs, sync protocol, local-first) — https://github.com/automerge/automerge
- Yjs README (CRDT shared types, offline editing, snapshots) — https://github.com/yjs/yjs
- Datomic overview (immutable facts/datoms, audit trail, as-of history) — https://docs.datomic.com/datomic-overview.html
- Zep docs overview (agent memory/context assembly) — https://docs.getzep.com/
- LlamaIndex docstores (document store backends and persistence) — https://docs.llamaindex.ai/en/stable/module_guides/storing/docstores/

---
*Feature research for: agentic doc stores / versioned document DBs*
*Researched: 2026-04-05*
