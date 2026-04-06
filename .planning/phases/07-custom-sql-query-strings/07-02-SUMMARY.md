---
phase: 07-custom-sql-query-strings
plan: 02
subsystem: database
tags: [postgres, sqlx, raw-sql, documentation]

# Dependency graph
requires:
  - phase: 07-01
    provides: typed query execution enum and repository raw-query fallback
provides:
  - bounded raw SQL execution in the Postgres adapter
  - real integration coverage for raw SQL read-only behavior and ceiling clamping
  - updated CLI/MCP query contract documentation
affects: [mcp, cli, query-contract, postgres-adapter]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - read-only raw SQL execution via transaction-level enforcement
    - JSON row materialization with `to_jsonb(...)`
    - server-side ceilings for rows and statement timeout

key-files:
  created: []
  modified:
    - crates/core/src/query.rs
    - crates/postgres/src/lib.rs
    - crates/postgres/tests/postgres_integration.rs
    - README.md
    - crates/mcp/README.md

key-decisions:
  - "Raw SQL takes precedence over guided query fields when `sql` is present."
  - "Raw SQL runs inside a read-only transaction and uses server-enforced ceilings of 100 rows and 5000ms."
  - "Raw rows are returned as JSON maps so the adapter never guesses column types."

patterns-established:
  - "Pattern 1: wrap caller SQL as a subquery and materialize rows with `to_jsonb(...)`."
  - "Pattern 2: clamp user-facing query knobs before execution instead of trusting request values."

requirements-completed: [TST-01, DOC-01]

# Metrics
duration: 6 min
completed: 2026-04-08
---

# Phase 07: custom-sql-query-strings Summary

**Bounded read-only raw SQL execution with real adapter coverage and public query docs**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-08T15:06:22Z
- **Completed:** 2026-04-08T15:11:52Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added raw SQL execution to the Postgres adapter with read-only transaction enforcement.
- Proved the path with integration tests for JSON row output, write rejection, and row-ceiling clamping.
- Documented the `sql` field precedence and the 100-row / 5000ms ceilings in the public docs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement bounded read-only raw SQL execution in the Postgres adapter** - `7437c99` (feat)
2. **Task 2: Prove the raw SQL path with real Postgres integration tests** - `3f646f0` (fix)
3. **Task 3: Document the sql field and ceiling rules** - `dfdc9fd` (docs)

## Files Created/Modified
- `crates/core/src/query.rs` - Carries raw SQL limit through parsing.
- `crates/postgres/src/lib.rs` - Executes bounded raw SQL in a read-only transaction.
- `crates/postgres/tests/postgres_integration.rs` - Covers raw SQL selection, write rejection, and limit clamping.
- `README.md` - Documents raw SQL precedence and ceilings for CLI users.
- `crates/mcp/README.md` - Documents the same query contract for MCP users.

## Decisions Made
- Raw SQL should remain a guided-query escape hatch, not a separate write-capable path.
- The adapter should clamp user input at execution time so ceilings are enforced even if callers overspecify values.
- JSON row materialization is the stable interface for raw SQL output.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Hardened raw SQL row materialization**
- **Found during:** Task 2 (integration test verification)
- **Issue:** The adapter briefly had a permissive fallback for non-object JSON rows.
- **Fix:** Reject non-object raw SQL row materialization as a storage error so the contract stays map-shaped.
- **Files modified:** `crates/postgres/src/lib.rs`
- **Verification:** `cargo test -p docracy-postgres --test postgres_integration raw_sql`
- **Committed in:** `3f646f0`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** No scope creep; the fix tightened the documented contract.

## Issues Encountered
- None.

## Known Stubs
- `README.md:319` contains a legacy TODO about linked documents. It is unrelated to raw SQL query execution and does not block this plan.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Raw SQL query execution is implemented, tested, and documented.
- The next plan can build on the same query contract without changing adapter ceilings.

## Self-Check: PASSED

---
*Phase: 07-custom-sql-query-strings*
*Completed: 2026-04-08*
