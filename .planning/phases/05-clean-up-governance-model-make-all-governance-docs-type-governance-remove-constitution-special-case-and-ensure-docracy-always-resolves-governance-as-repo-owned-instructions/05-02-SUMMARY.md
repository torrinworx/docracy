---
phase: 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions
plan: 02
subsystem: infra
tags: [governance, cli, mcp, rust, docs]

# Dependency graph
requires:
  - phase: 05-01
    provides: governance doc type rename and stored-row migration baseline
provides:
  - fixed repo-owned governance bundle helper
  - startup surfaces locked to `./governance`
  - docs that describe governance type/path contract
affects: [CLI startup, MCP startup, public docs]

# Tech tracking
tech-stack:
  added: []
  patterns: [repo-owned startup path helper, interface-level governance bootstrap]

key-files:
  created: []
  modified: [crates/core/src/governance.rs, crates/cli/src/main.rs, crates/mcp/src/config.rs, crates/mcp/src/runtime.rs, crates/mcp/src/bin/docracy-mcp.rs, README.md, crates/mcp/README.md]

key-decisions:
  - "Centralize the fixed bundle path in `FsGovernanceSource::repo_owned()` so CLI and MCP share one startup source of truth."
  - "Keep the repo-owned bundle path relative as `./governance` rather than adding new path configuration."

patterns-established:
  - "Pattern 1: Startup config no longer accepts a user-provided governance directory."
  - "Pattern 2: Public docs mirror the exact shipped CLI/MCP governance-path contract."

requirements-completed: [GOV-07, DOC-03]

# Metrics
duration: 5min
completed: 2026-04-06
---

# Phase 5: Fixed Governance Startup Summary

CLI and MCP now always boot from the repo-owned `./governance` bundle, and public docs describe the same fixed contract.

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-06T15:44:40-04:00
- **Completed:** 2026-04-06T15:44:47-04:00
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added a repo-owned governance helper and removed user-configurable governance path fields from startup surfaces.
- Wired CLI and MCP bootstrap to the same fixed `./governance` bundle.
- Updated public docs to name the governance doc type and fixed bundle path.

## Task Commits

1. **Task 1: Freeze CLI and MCP startup on the repo-owned governance bundle** - `1da5e29` (fix)
2. **Task 2: Refresh public docs for the governance type and fixed path** - `c7c5527` (docs)

## Files Created/Modified
- `crates/core/src/governance.rs` - fixed repo-owned helper
- `crates/cli/src/main.rs` - init command now always uses the repo-owned bundle
- `crates/mcp/src/config.rs` - removed `governance_dir` from startup config
- `crates/mcp/src/runtime.rs` - bootstrap now always uses the repo-owned bundle
- `crates/mcp/src/bin/docracy-mcp.rs` - removed path override from the binary flags
- `README.md` - public contract now describes governance docs and the fixed bundle path
- `crates/mcp/README.md` - MCP startup notes now reflect the fixed path contract

## Decisions Made
- Exposed a small `FsGovernanceSource::repo_owned()` helper instead of passing path overrides through startup config.
- Treated `./governance` as the fixed repo-owned path in both CLI and MCP entrypoints.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered
- None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Startup path handling is now repo-defined and consistent across interfaces.
- Documentation now matches the shipped governance contract.

---
*Phase: 05-clean-up-governance-model-make-all-governance-docs-type-governance-remove-constitution-special-case-and-ensure-docracy-always-resolves-governance-as-repo-owned-instructions*
*Completed: 2026-04-06*

## Self-Check: PASSED
