# Requirements: Docracy

**Defined:** 2026-04-05
**Core Value:** Agents can safely manage durable, queryable long-term memory without losing history, context, or control over document structure.

## v1 Requirements

### Core Document Model

- [ ] **CORE-01**: User can create a typed document with content and metadata
- [ ] **CORE-02**: User can read a document by id and inspect its current revision

### Revisioning

- [ ] **REV-01**: Updating a document creates a new revision and archives the prior revision
- [ ] **REV-02**: User can inspect revision history for a document

### Retrieval

- [ ] **READ-01**: User can search documents by keyword
- [ ] **READ-02**: User can filter documents by type, status, date, and metadata
- [ ] **READ-03**: User can retrieve related documents through stored references

### Governance

- [ ] **GOV-01**: The system loads seed constitution and context documents on init
- [ ] **GOV-02**: Write operations can be validated or blocked by policy rules

### Interfaces

- [ ] **INT-01**: The core document engine can be exercised directly through tests
- [ ] **INT-02**: A CLI can create, read, update, archive, and search documents
- [ ] **INT-03**: A service or MCP adapter can call the same core operations

## v2 Requirements

### Extensibility

- **EXT-01**: Users can define new metadata fields with controlled indexing rules
- **EXT-02**: Users can assign documents to clients or workspaces
- **EXT-03**: Users can validate updates with pluggable document validators

### Retrieval Enhancements

- **SRCH-01**: Semantic ranking can augment structured search results
- **SRCH-02**: Search can span archived history when explicitly requested

## Out of Scope

| Feature | Reason |
|---------|--------|
| Git as the system of record | The project is intentionally separate from git workflows |
| Programming-only scope | The system is meant to work for general document management |
| Live multi-user collaboration UI | The first deliverable is the core engine and thin interfaces |
| Prompt-injection-proof validator agents | Interesting later, but not a safe v1 guarantee |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Pending |
| CORE-02 | Phase 1 | Pending |
| REV-01 | Phase 1 | Pending |
| REV-02 | Phase 1 | Pending |
| READ-01 | Phase 2 | Pending |
| READ-02 | Phase 2 | Pending |
| READ-03 | Phase 2 | Pending |
| GOV-01 | Phase 3 | Pending |
| GOV-02 | Phase 3 | Pending |
| INT-01 | Phase 4 | Pending |
| INT-02 | Phase 4 | Pending |
| INT-03 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 12 total
- Mapped to phases: 12
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-05*
*Last updated: 2026-04-05 after initialization*
