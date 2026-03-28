# Pitfalls Research

**Domain:** Agentic document store / versioned document database for LLM agents (Postgres-backed, typed docs + status + full revision history)
**Researched:** 2026-04-05
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: Blurring *document identity* vs *revision identity*

**What goes wrong:**
Teams store “a document” as a single mutable row, then tack on history later (or keep history but still allow in-place updates). Auditability, diffing, rollback, and reproducibility become unreliable; bugs or agent mistakes can silently rewrite the past.

**Why it happens:**
CRUD instincts + pressure to ship a simple API. “We can add versioning later” feels easy until other systems start depending on stable revision IDs.

**How to avoid:**
- Make **doc_id** stable and **revision_id** immutable.
- Enforce **append-only revisions** at the DB/API layer (no UPDATE of revision content; only new revisions).
- Store **parent_revision_id** (or a revision sequence) and make “latest” a derived concept.
- Require every read to specify either `doc_id@latest` or an explicit `revision_id`.

**Warning signs:**
- Bugs described as “history looks weird” or “diff is wrong”
- Multiple code paths writing to the same table with UPDATEs
- “Latest revision” stored as a mutable pointer without constraints

**Phase to address:**
Foundation (data model invariants + storage schema)

---

### Pitfall 2: No concurrency story for multi-agent edits (lost updates)

**What goes wrong:**
Two agents read the same revision and both write a new revision; one overwrites the other (or “last write wins” hides the conflict). This is especially destructive for governance docs and long-lived project context.

**Why it happens:**
Agent tool calls are often retried and parallelized; optimistic concurrency is “later.”

**How to avoid:**
- Use **optimistic concurrency**: updates must include `expected_parent_revision_id` (or expected revision sequence). Reject if head moved.
- Provide an explicit **conflict surface**: return `409 Conflict` with both head revisions; do not auto-merge silently.
- If you support auto-merge later, make it opt-in and type-specific.

**Warning signs:**
- Agents complain “my changes disappeared”
- Unexpected oscillation in content (“undo/redo” style) across revisions
- Frequent head changes without corresponding agent awareness

**Phase to address:**
Core API/CLI contract (update semantics + errors) + Foundation (revision graph design)

---

### Pitfall 3: Using wall-clock timestamps as the source of truth for ordering

**What goes wrong:**
Revision ordering becomes non-deterministic under clock skew, retries, or async ingestion. “Latest” queries return surprising results, and replay/debug becomes flaky.

**Why it happens:**
Timestamps are convenient and familiar; teams underestimate distributed/parallel writers (even on one machine, you can have concurrent processes).

**How to avoid:**
- Define ordering by **(doc_id, monotonic revision sequence)** or by a **strict parent pointer**.
- Keep timestamps as metadata, not ordering.
- Ensure every new revision is created in a single DB transaction that assigns the next sequence / validates parent.

**Warning signs:**
- “Newest revision sometimes isn’t newest”
- Queries that sort by `created_at` but are used for correctness

**Phase to address:**
Foundation (revision ordering invariants)

---

### Pitfall 4: Treating typed documents as “just JSON/text” (no schema/version strategy)

**What goes wrong:**
You ship typed docs, then later change the schema for a doc type and can’t deserialize old revisions, can’t run migrations safely, or can’t query reliably across time.

**Why it happens:**
Schema evolution is easy to ignore early; “it’s just docs” until governance and tools depend on structure.

**How to avoid:**
- Include a **doc_type_version** (per type) in each revision.
- Use **tolerant parsing**: ignore unknown fields, default missing fields.
- Treat breaking changes explicitly: upcasters at read time, or one-time backfill jobs.
- Never assume “current schema” can read “all history” without a plan.

**Warning signs:**
- “Old docs crash the CLI/tool”
- Type-specific queries silently skip older revisions
- One-off ad-hoc scripts to “fix old docs”

**Phase to address:**
Storage & migrations (type/version discipline) + Query/Search (cross-version behavior)

---

### Pitfall 5: Status fields without a state machine (workflow drift)

**What goes wrong:**
Agents set impossible statuses (“approved” without “reviewed”), or different agents invent new statuses. Query filters become meaningless; humans stop trusting the store.

**Why it happens:**
Status feels like “a string column” until you need governance.

**How to avoid:**
- Define **allowed statuses per doc type** and allowed transitions (state machine).
- Log **status transition events** as part of the revision metadata.
- Provide tool-level helpers: `advance_status(doc_id, transition)` not `set_status("whatever")`.

**Warning signs:**
- Many near-duplicate statuses ("archived", "archive", "inactive")
- Agents frequently query without status filters because filters are unreliable

**Phase to address:**
Governance seed + core schema (doc types/status invariants)

---

### Pitfall 6: Non-idempotent tool calls (duplicate revisions / duplicate seeding)

**What goes wrong:**
Retries (agent loops, CLI retries, network hiccups) create duplicate documents or duplicate revisions, breaking audit and confusing retrieval.

**Why it happens:**
Agent tool ecosystems are “at least once” by default; idempotency is often forgotten.

**How to avoid:**
- Require an **idempotency key** per Create/Update (per actor/session/tool-call).
- Make seeding **re-runnable**: detect existing seed docs by stable IDs + content hash, then no-op.
- Store `(actor_id, idempotency_key)` with a unique constraint.

**Warning signs:**
- Two identical revisions with same content created seconds apart
- Seed docs duplicated across initializations

**Phase to address:**
Core API/CLI contract + DB constraints

---

### Pitfall 7: “Latest-only” retrieval without a principled definition (stale/archived leaks)

**What goes wrong:**
Queries return deleted/archived docs, or return an older revision because “latest” wasn’t computed correctly. Agents then act on stale governance/context.

**Why it happens:**
Teams implement search and filtering early but forget to encode domain rules: latest non-deleted revision, status constraints, visibility.

**How to avoid:**
- Define and enforce a single rule: **visible head revision** = latest revision that is not deleted + matches visibility policy.
- Implement “head” as a **DB view** or query helper used everywhere (no copy/paste SQL).
- Provide explicit query modes: `latest_visible`, `latest_any`, `as_of(revision_id)`, `history`.

**Warning signs:**
- Different commands return different content for the “same doc”
- Bug reports: “search finds it but read doesn’t”

**Phase to address:**
Query/Search primitives (views, filters, pagination)

---

### Pitfall 8: Ignoring retention, redaction, and “right to be forgotten” in an append-only history

**What goes wrong:**
Secrets/PII get into revisions and become effectively permanent. Later, compliance/security requires deletion, but hard-deleting breaks referential integrity and audit trails.

**Why it happens:**
Versioned stores feel like “immutable truth,” but regulations and operational security require selective erasure.

**How to avoid:**
- Design “delete” as policy-driven: **soft delete / archive** for normal lifecycle; **redaction workflows** for secrets/PII.
- Keep sensitive payloads **out of the immutable log** when possible (store references; or encrypt with per-subject keys and support crypto-shredding).
- Build a **redaction event** that preserves structure but removes content.

**Warning signs:**
- People paste API keys into docs; no remediation path
- You can’t answer “what data do we retain and for how long?”

**Phase to address:**
Security & governance (data classification, retention, redaction mechanics)

---

### Pitfall 9: Constitution/governance drift between repo and DB (or agents mutating the constitution)

**What goes wrong:**
The repo-owned immutable constitution is seeded once, then diverges in the DB, or an agent “updates governance” by editing it in the DB. Agents now follow inconsistent rules.

**Why it happens:**
Seeding is treated as “initial insert,” not as an ongoing integrity guarantee.

**How to avoid:**
- Treat constitution as **code-owned**: store a **content hash/version** in DB, verify on startup/Init.
- Disallow mutation: mark constitution docs as **write-protected** at the API level (and ideally via DB constraints).
- Provide a deliberate workflow for governance updates: “bump constitution version in repo → migration applies → DB reflects.”

**Warning signs:**
- DB copy of constitution differs from repo file
- Agents can update constitution via the same Update tool used for normal docs

**Phase to address:**
Governance seeding + API authorization rules

---

### Pitfall 10: Allowing ungoverned “extensions” to become a de facto schema (slow queries + instability)

**What goes wrong:**
Teams start querying JSONB extensions ad-hoc (“just filter on extensions.foo”), creating fragile coupling and unindexed scans. Agents also stuff large/hostile payloads into extensions, inflating storage and slowing everything.

**Why it happens:**
Extensions are tempting as “flexible metadata” before policies exist.

**How to avoid:**
- Keep extensions **write-validated** (size limits, allowed keys, per-doc-type policies).
- Don’t expose arbitrary extension querying until governance defines a stable contract.
- If you must query, require **declared indexes** (generated columns / GIN paths) and a schema registry for extension keys.

**Warning signs:**
- More and more product features depend on extension keys
- Query plans show sequential scans over large JSONB

**Phase to address:**
Governance + Query/Search (index strategy) — explicitly defer until policies exist

---

### Pitfall 11: Treating derived indexes as canonical (search index drift)

**What goes wrong:**
Full-text search results don’t match the source of truth because indexing is async or lossy, and rebuild/backfill isn’t supported. Agents “trust search” and act on nonexistent/stale docs.

**Why it happens:**
Search is added as a feature without lifecycle tooling: backfills, reindexing, verification.

**How to avoid:**
- Make the **revision store canonical**; make search explicitly **derived**.
- Support **reindex from source** (full rebuild) and incremental indexing.
- Add a cheap consistency check: sample documents where `search_vector`/index version mismatches.

**Warning signs:**
- “Search finds ghost docs” or “search misses recent updates”
- No operational command to rebuild indexes

**Phase to address:**
Query/Search + Ops tooling

---

### Pitfall 12: Not designing for agent-security realities (memory poisoning + tool abuse)

**What goes wrong:**
Attackers (or just untrusted retrieved content) cause agents to store malicious instructions as durable “memory”/docs, which later influence decisions. Over-permissioned tools let compromised agents write/erase governance or exfiltrate data.

**Why it happens:**
Teams treat the doc store as “internal,” but agents ingest untrusted text and can be socially engineered.

**How to avoid:**
- Apply **least privilege**: split read vs write tools; limit write scopes by doc type/status.
- Validate/sanitize before persistence (especially for docs that will be retrieved into prompts).
- Add **memory isolation** boundaries (project/workspace/user) and explicit trust levels per document.
- Implement monitoring for abnormal write patterns (DoW/looping) and add circuit breakers.

**Warning signs:**
- Documents contain “instructions to the agent” embedded in untrusted content
- Sudden spike in writes/updates without corresponding user intent
- Agents attempt to modify governance docs

**Phase to address:**
Security & governance + Observability

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| No branching history (force linear parent pointer only) | Simpler API/UX | Harder to model true concurrent edits; conflict handling becomes manual | Acceptable for v1 **if** conflicts are surfaced (409) and tooling exists to reconcile |
| “Latest” computed in application code | Faster to ship | Divergent semantics across commands; bugs around deleted/archived docs | Never (centralize as view/helper) |
| Free-form statuses | Zero governance work | Workflow drift; unreliable queries | MVP only if you lock statuses quickly and migrate |
| No idempotency key | Less plumbing | Duplicate writes on retries; hard-to-debug state | Never for Create/Update tools in agent ecosystems |
| Index/search as best-effort without rebuild command | Quick demo | Permanent drift and trust loss | MVP only if you also ship reindex/backfill early |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Agent tool callers (LLM frameworks) | Assuming exactly-once tool execution | Design for **at-least-once**: idempotency keys + conflict-aware updates |
| CLI as first interface | Making CLI the only validator | Enforce invariants in the Rust core + DB constraints; CLI is just a client |
| “Init” seeding | Writing seeds blindly every time | Seed with stable IDs + content hash; verify constitution hash/version |
| Future vector layer | Storing embeddings without revision linkage | Bind derived artifacts to `(doc_id, revision_id, embed_model_id)`; support invalidation |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| History table grows unbounded without retention/partitioning strategy | Slow queries, bloated indexes, backups hurt | Partition by time/doc_type; archive policies; vacuum/index maintenance | Typically at “months of heavy agent use” rather than user count |
| “Latest” query uses correlated subqueries everywhere | High CPU, slow pagination | Maintain head pointers with constraints, or materialized/latest view with indexes | At moderate doc counts (10^5–10^6 revisions) |
| Full-text index updated synchronously on every write | High write latency | Async indexing or computed columns with careful indexing; batch rebuild | When agents do many rapid updates |
| Filtering by JSONB extensions without indexes | Seq scans, timeouts | Limit/query only indexed keys; add generated columns | As soon as extensions become common |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Allowing writes to governance/constitution via the same Update tool | Governance takeover | Write-protect by doc type + DB constraints; repo-hash verification |
| Storing untrusted retrieved text as durable “memory” without validation | Memory poisoning / goal hijack | Sanitize + classify docs; isolate trust levels; retrieval-time filtering |
| No deletion/redaction path for secrets/PII | Compliance + incident response failure | Retention + redaction workflows; keep PII out of immutable history when possible |
| Over-permissioned tools (wildcard query/update) | Data exfiltration, corruption | Least privilege + scoped capabilities + approvals for high-risk actions |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| “Update” silently succeeds but creates a conflict revision or duplicate | Users/agents don’t understand state | Make conflicts explicit (409) and provide a reconcile workflow |
| No simple way to answer “why did this change?” | Audit trail unused | Store `actor_id`, `reason`, `tool_call_id`, and show diffs between revisions |
| Search results not explainable (“why did this doc match?”) | Agents over-trust retrieval | Provide highlight/snippets, ranking rationale (BM25 terms), and filter visibility |

## "Looks Done But Isn't" Checklist

- [ ] **Versioning:** No UPDATEs to immutable revision content; parent pointer enforced; reads can pin `revision_id`.
- [ ] **Concurrency:** Update requires expected parent/head; conflicts are visible and test-covered.
- [ ] **Idempotency:** Create/Update accept idempotency keys and have unique constraints.
- [ ] **Query correctness:** One canonical definition of “latest visible revision,” used everywhere.
- [ ] **Governance integrity:** Constitution hash/version verified; constitution write-protected.
- [ ] **Retention/redaction:** There is a defined process to remove secrets/PII without corrupting history.
- [ ] **Search lifecycle:** Reindex/backfill command exists; drift can be detected.
- [ ] **Security boundaries:** Tool permissions are scoped; untrusted content is treated as untrusted.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Blurred doc vs revision identity | HIGH | Freeze writes; migrate to append-only revisions; backfill revision table; re-derive heads + indexes |
| Lost updates from concurrency | MEDIUM/HIGH | Add optimistic concurrency; identify conflicting edits; manual reconcile tooling; communicate semantics to agents |
| Schema evolution breakage | MEDIUM | Add versioning + tolerant parsing; write upcasters; backfill critical docs; add tests on old revisions |
| Search/index drift | LOW/MEDIUM | Rebuild index from source-of-truth; add drift checks; make indexing incremental with checkpoints |
| Secrets/PII in immutable history | HIGH | Incident response: rotate leaked secrets; redact content with dedicated event; implement crypto-shredding / externalize sensitive fields |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Doc vs revision identity | Foundation | DB forbids UPDATE on revisions; tests assert immutable history; reads can pin revision |
| Multi-agent concurrency | Core API/CLI | 409 conflict test; concurrent writers test; conflict surfaced with both heads |
| Ordering by timestamps | Foundation | “latest” defined by sequence/parent; property tests for ordering |
| Schema evolution | Storage & migrations | Can read old revisions after schema changes; upcaster tests |
| Status drift | Governance | Status transition validation tests; no unknown statuses in DB |
| Non-idempotent tools | Core API/CLI | Retry same tool call produces same result; unique constraint enforced |
| Latest retrieval correctness | Query/Search | Search/read consistency tests; archived/deleted never returned in latest_visible |
| Retention/redaction | Security & governance | Redaction workflow test; documented retention policy; audit of sensitive fields |
| Constitution drift | Governance | Repo hash check fails loud; DB constitution matches repo; write attempts rejected |
| Extensions as schema | Governance + Query | Extension size/keys validated; no ad-hoc JSONB scans in query plans |
| Derived index drift | Query/Search + Ops | Reindex command; drift detection metric; rebuild verified |
| Agent-security realities | Security + Observability | Least-privilege tool tests; injection/memory-poisoning regression tests; anomaly alerts |

## Sources

- Microsoft Learn (Azure Architecture Center), **Event Sourcing pattern** (updated 2026-03-27): schema evolution, ordering, idempotency, and regulatory deletion tension in append-only histories. https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing
- OWASP Cheat Sheet Series, **AI Agent Security Cheat Sheet**: memory poisoning, tool least privilege, multi-agent security, denial-of-wallet, logging/observability. https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- Wang et al., **Unveiling Privacy Risks in LLM Agent Memory** (arXiv:2502.13172, ACL 2025): demonstrates practical memory-extraction attacks; motivates safeguards for durable agent memory stores. https://arxiv.org/abs/2502.13172
- (Future vector milestone; LOW confidence citation use) Pinecone, **Accurate and Efficient Metadata Filtering in Pinecone's Serverless Vector Database** (ICML 2025 paper): indicates metadata filtering is a core correctness/perf concern for RAG retrieval. https://www.pinecone.io/research/ICML_2025.pdf

---
*Pitfalls research for: agentic doc store / versioned document DB*
*Researched: 2026-04-05*
