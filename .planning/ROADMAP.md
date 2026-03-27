# Roadmap: Docracy

**Created:** 2026-04-05
**Granularity:** Standard
**Execution:** Parallel where independent, sequential where blocked by core dependencies

## Summary

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 1 | Core Engine and Revisions | Establish the document model and history model | CORE-01, CORE-02, REV-01, REV-02 | 4 |
| 2 | Search and Retrieval | Make stored documents discoverable through structured search | READ-01, READ-02, READ-03 | 3 |
| 3 | Governance and Init | Load governance documents and enforce write policy | GOV-01, GOV-02 | 3 |
| 4 | CLI and Test Harness | Expose the core through tests and a minimal command surface | INT-01, INT-02 | 4 |
| 5 | Service and Adapter Surface | Add the network/MCP adapter on top of the same core | INT-03 | 3 |

## Phase Details

### Phase 1: Core Engine and Revisions

**Goal:** Create the durable document model, typed metadata, and revision handling.

**Requirements:** CORE-01, CORE-02, REV-01, REV-02

**Success criteria:**
1. A document can be created with type, content, and metadata.
2. Reading a document returns the current revision and metadata.
3. Updating a document writes a new revision and archives the old one.
4. Revision history is queryable and preserves prior content.

### Phase 2: Search and Retrieval

**Goal:** Make documents easy to discover through structured and relation-aware queries.

**Requirements:** READ-01, READ-02, READ-03

**Success criteria:**
1. Keyword search returns relevant documents.
2. Filters can combine type, status, date, and metadata fields.
3. Related documents can be retrieved through stored references.

### Phase 3: Governance and Init

**Goal:** Ensure the system can load seed governance context and enforce write rules.

**Requirements:** GOV-01, GOV-02

**Success criteria:**
1. Init loads constitution and context seed documents.
2. Policy hooks can block invalid writes before persistence.
3. Governance behavior is covered by tests.

### Phase 4: CLI and Test Harness

**Goal:** Provide the first usable interface for exercising the core engine.

**Requirements:** INT-01, INT-02

**Success criteria:**
1. Core operations are testable directly without an interface layer.
2. The CLI can create, read, update, archive, and search documents.
3. CLI commands call the same core paths as tests.
4. Interface behavior matches core invariants.

### Phase 5: Service and Adapter Surface

**Goal:** Add a service or MCP adapter without changing core behavior.

**Requirements:** INT-03

**Success criteria:**
1. The adapter can call the same document operations as the CLI.
2. Network exposure does not bypass validation or revision rules.
3. Adapter tests confirm parity with the core contract.

## Coverage

- v1 requirements: 12
- Mapped to phases: 12
- Unmapped: 0

## Build Order

1. Finish the core engine and revision model.
2. Add search and retrieval.
3. Add governance and init behavior.
4. Wrap the core in tests and CLI.
5. Add network or MCP access last.
