---
phase: 01-mcp-crate-interface-boundary
plan: 01
subsystem: api
tags: [rust, mcp, postgres, sqlx]

# Dependency graph
requires:
  - phase: v1.0
    provides: Shipped core use-cases + Postgres adapter + thin CLI boundary
provides:
  - Dedicated `docracy-mcp` interface crate in the workspace
  - MCP-owned startup config model (DB URL, governance path, migration policy, transport selection)
  - Transport-agnostic runtime/bootstrap helper wiring core dependencies
affects: [phase-02-tool-surface-stdio, phase-03-streamable-http]

# Tech tracking
tech-stack:
  added: [docracy-mcp]
  patterns:
    - Library-first interface crate that delegates to `docracy-core`
    - Shared runtime/bootstrap module for multi-transport reuse

key-files:
  created:
    - crates/mcp/Cargo.toml
    - crates/mcp/src/lib.rs
    - crates/mcp/src/config.rs
    - crates/mcp/src/runtime.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - .planning/codebase/ARCHITECTURE.md

key-decisions:
  - "Keep `crates/mcp` library-first and transport-agnostic; stdio/http servers wrap shared bootstrap later"
  - "Own runtime/config in `docracy-mcp` and delegate domain logic to `docracy-core` use-cases"

patterns-established:
  - "Interface boundary: request/response shaping + error mapping in interface crates; invariants in core"
  - "Single bootstrap path initializes PgRepository + governance + clock/ids for reuse"

requirements-completed: [IFC-01, CFG-01, DOC-02]

# Metrics
duration: 1 min
completed: 2026-04-06
---

# Phase 1 Plan 01: MCP crate bootstrap + runtime/config boundary Summary

**A dedicated `docracy-mcp` workspace crate with a reusable startup config + runtime/bootstrap boundary for future stdio and Streamable HTTP transports.**

## Performance

- **Duration:** 1 min
- **Started:** 2026-04-05T20:51:25-04:00
- **Completed:** 2026-04-05T20:52:50-04:00
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments
- Added `crates/mcp` as a first-class workspace crate (`docracy-mcp`).
- Defined MCP-owned startup config (DB URL, governance dir, migration policy, transport selection).
- Implemented transport-agnostic runtime bootstrap wiring `PgRepository`, `FsGovernanceSource`, `SystemClock`, and `UuidV4Generator`.
- Documented the intended interface boundary so business rules remain single-sourced in `docracy-core`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the workspace-level MCP crate scaffold** - `11cbb21` (feat)
2. **Task 2: Define MCP-owned startup config and reusable runtime bootstrap** - `36fa6d0` (feat)
3. **Task 3: Document the MCP interface boundary for contributors** - `d33d773` (docs)

## Files Created/Modified
- `Cargo.toml` - Adds `crates/mcp` to the workspace.
- `crates/mcp/Cargo.toml` - New `docracy-mcp` interface crate manifest.
- `crates/mcp/src/config.rs` - Startup config model including transport selection.
- `crates/mcp/src/runtime.rs` - Shared bootstrap helper (connect + optional migrations + dependency wiring).
- `.planning/codebase/ARCHITECTURE.md` - Documents `crates/mcp` role and the core-vs-interface boundary.

## Decisions Made
- Keep MCP startup/runtime concerns in `docracy-mcp` so transports can share one initialization path.
- Enforce the interface boundary in docs: business rules stay in `docracy-core` and MCP delegates to core use-cases.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `gsd-tools roadmap update-plan-progress` incorrectly marked Phase 1 plan 01-02 as completed; corrected `.planning/ROADMAP.md` manually to reflect 01-01 complete and 01-02 pending.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 can implement MCP tool registration and stdio transport by calling the shared `docracy-mcp` bootstrap.
- Architecture notes now explicitly discourage duplicating business rules in the MCP interface layer.

## Self-Check: PASSED

- FOUND: .planning/phases/01-mcp-crate-interface-boundary/01-01-SUMMARY.md
- FOUND commits: 11cbb21, 36fa6d0, d33d773
