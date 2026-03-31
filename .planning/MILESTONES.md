# Milestones

## v1.0 MVP (Shipped: 2026-04-05)

**Phases completed:** 4 phases, 7 plans, 16 tasks

**Key accomplishments:**

- Revision-safe document updates now require an expected head revision across the core service, CLI input, and Postgres adapter, with stale writes rejected before new revisions are chained.
- CLI update payloads now use expected_revision, errors emit machine-readable JSON, and README examples match the v1 query contract without implying extension search support.
- Reusable deterministic core fixtures now cover revision chaining, init repair, and query semantics directly inside `docracy_core`, without depending on the CLI.
- Postgres-backed integration tests now validate schema migrations, constitution repair, and query/search behavior through an isolated adapter harness.
- Postgres revision-lineage guards, a fixed `migrate` override, and contract-aligned docs harden the v1 core against the last validation gaps.
- Dedicated verification artifacts now exist for the completed validation and stabilization phases, and the CLI's structured stderr contract is pinned by a black-box regression fixture.

---
