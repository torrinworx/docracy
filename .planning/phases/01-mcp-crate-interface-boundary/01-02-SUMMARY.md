---
phase: 01-mcp-crate-interface-boundary
plan: 02
subsystem: api
tags: [rust, mcp, delegation, error-mapping]

# Dependency graph
requires:
  - phase: 01-mcp-crate-interface-boundary/01-01
    provides: MCP crate scaffold + runtime/config bootstrap
provides:
  - Thin MCP-facing operations layer delegating to core use-cases
  - Interface-local, machine-readable error mapping for core/setup failures
  - In-memory regression tests locking delegation + conflict mapping
affects: [phase-02-tool-surface, phase-03-http-transport, testing]

# Tech tracking
tech-stack:
  added: [serde, serde_json]
  patterns: ["dependency-injected operations for testability", "interface-local error mapping"]

key-files:
  created:
    - crates/mcp/src/operations.rs
    - crates/mcp/src/error.rs
    - crates/mcp/tests/operations.rs
  modified:
    - crates/mcp/src/lib.rs
    - crates/mcp/Cargo.toml
    - Cargo.lock

key-decisions:
  - "Keep MCP-facing errors in crates/mcp (McpErrorKind + optional JSON details) to avoid contaminating docracy_core with protocol concerns."
  - "Make operations accept core traits (Repository/Clock/IdGenerator) and provide runtime convenience wrappers so tests can use MemoryRepository without transports."

patterns-established:
  - "Boundary pattern: crates/mcp/src/operations.rs delegates to docracy_core::* use-cases; crates/mcp/src/error.rs maps CoreError into stable kinds/details."

requirements-completed: [IFC-02, IFC-03]

# Metrics
duration: 4 min
completed: 2026-04-06
---

# Phase 01 Plan 02: Core Delegation Layer + MCP Error Mapping Summary

**A thin, test-locked MCP interface boundary that delegates Init/Create/Read/Query/Update to `docracy_core` and maps `CoreError` into stable MCP-facing kinds with structured conflict details.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-05T21:02:42-04:00
- **Completed:** 2026-04-05T21:05:58-04:00
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added `crates/mcp/src/operations.rs` as the reusable Docracy operation layer for future MCP tool handlers.
- Implemented interface-local `McpError` / `McpErrorKind` mapping that preserves `revision_conflict` expected/actual details.
- Locked the delegation boundary with in-memory integration tests that require no transport.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the thin MCP operation/delegation layer** - `24f0500` (feat)
2. **Task 2: Add stable MCP-facing response and error mapping** - `f350b66` (feat)
3. **Task 3: Lock the delegation boundary with focused crate tests** - `adfebb8` (test)

## Files Created/Modified

- `crates/mcp/src/operations.rs` - Dependency-injected operation helpers + runtime convenience wrappers delegating to `docracy_core`.
- `crates/mcp/src/error.rs` - MCP-facing error translation (`CoreError` → stable kind/message/details).
- `crates/mcp/tests/operations.rs` - In-memory regression tests for delegation + conflict detail mapping.
- `crates/mcp/src/lib.rs` - Export MCP-facing modules/types used by later plans.
- `crates/mcp/Cargo.toml` / `Cargo.lock` - Add serde/serde_json for error details and tokio dev-dep for async tests.

## Decisions Made

- Used a stable `McpErrorKind` enum with optional JSON `details` to keep the interface machine-readable while remaining protocol-agnostic.
- Refactored operations helpers to accept core trait dependencies (Repository/Clock/IdGenerator) so tests can use `MemoryRepository` deterministically.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added an early integration test target + tokio dev-dependency to satisfy per-task verification**
- **Found during:** Task 1 (verification requires `cargo test -p docracy-mcp --test operations`)
- **Issue:** The plan’s per-task verification command requires an `operations` integration test target, which did not exist yet.
- **Fix:** Created `crates/mcp/tests/operations.rs` early (smoke), added `tokio` as a dev-dependency so async tests could run, then replaced the smoke test with real in-memory coverage in Task 3.
- **Files modified:** `crates/mcp/tests/operations.rs`, `crates/mcp/Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo test -p docracy-mcp --test operations`
- **Committed in:** `24f0500` (and expanded in `adfebb8`)

**2. [Rule 3 - Blocking] Avoided gsd-tools phase auto-resolution for phase updates due to v1.0/v1.1 name collision**
- **Found during:** Plan execution (init resolved phase `01` to the archived v1.0 directory)
- **Issue:** `gsd-tools init execute-phase` resolved to `.planning/phases/01-cli-mvp-core-finalization` instead of `.planning/phases/01-mcp-crate-interface-boundary`.
- **Fix:** Performed ROADMAP/STATE updates manually by editing the explicit files on disk rather than using numeric phase resolution.
- **Files modified:** `.planning/ROADMAP.md`, `.planning/STATE.md`
- **Verification:** N/A (documentation-only update)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required to run the plan’s verification command and to avoid corrupting milestone tracking files. No business logic scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 MCP interface boundary is stable and regression-covered.
- Ready for Phase 2 plan 02-01 (tool registration + schemas + handler coverage).

## Self-Check: PASSED

- Confirmed `.planning/phases/01-mcp-crate-interface-boundary/01-02-SUMMARY.md` exists
- Confirmed task commits exist: `24f0500`, `f350b66`, `adfebb8`
