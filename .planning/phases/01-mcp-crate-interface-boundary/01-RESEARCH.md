# Phase 1: MCP Crate + Interface Boundary - Research

**Researched:** 2026-04-06
**Domain:** Rust MCP server crate design over an existing core + Postgres adapter
**Confidence:** HIGH

<user_constraints>
## User Constraints

### Locked Decisions
- `crates/mcp` must be a separate interface crate alongside `crates/cli`.
- MCP-facing operations must reuse `docracy_core` use-cases instead of reimplementing business logic.
- Postgres connection setup, governance path, migration behavior, and transport selection belong in the interface layer.
- Error mapping and response shaping belong in the MCP crate, while domain invariants remain in the core.

### the agent's Discretion
- Exact internal module layout for config/runtime/delegation helpers.
- The exact names of response/error wrapper types.

### Deferred Ideas (OUT OF SCOPE)
- MCP tool registration and schemas
- Stdio transport delivery
- Streamable HTTP delivery
- OAuth, prompts, resources, sampling, subscriptions

</user_constraints>

<research_summary>
## Summary

Phase 1 should establish the MCP layer as a normal Rust interface crate, not a second implementation path. The existing Docracy codebase is already shaped for this: `docracy_core` exports the business use-cases, `docracy_postgres` owns the concrete repository adapter, and the CLI demonstrates the current boundary pattern for startup, delegation, and user-facing error shaping.

The official Rust MCP SDK is `rmcp`. The right architecture for Docracy is one reusable handler/delegation layer inside `crates/mcp`, with transports added around it later. For this phase, that means building the crate, a transport-agnostic config/runtime bootstrap, and MCP-facing operation/error mapping that future stdio and HTTP entrypoints can share.

**Primary recommendation:** Build `crates/mcp` now as a thin library-first adapter with reusable startup and core-delegating operations; postpone concrete transport serving until phases 2 and 3.

</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 1.x line active; validate exact API at implementation time | Official Rust MCP SDK | Gives a supported MCP server foundation instead of a custom protocol stack |
| `tokio` | existing workspace baseline | Async runtime for MCP server startup and Postgres I/O | Already used by the CLI and Postgres adapter |
| `docracy-core` | workspace crate | Canonical business use-cases | Keeps document/governance behavior single-sourced |
| `docracy-postgres` | workspace crate | Concrete Postgres repository + migrations | Reuses the shipped storage adapter rather than rebuilding setup logic |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` / `serde_json` | existing workspace baseline | Request/response structs and error details | For MCP-facing payloads and detail objects |
| `thiserror` or existing typed error approach | implementation choice | Interface-local error mapping | Useful if the MCP crate grows enough error cases to justify a local enum |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rmcp` | Hand-rolled JSON-RPC/MCP glue | Faster to start poorly, slower to maintain correctly |
| Thin adapter over `docracy_core` | Duplicate logic in `crates/mcp` | Creates behavior drift between CLI and MCP |
| Shared runtime/bootstrap module in `crates/mcp` | Per-transport setup code | Leads to duplicated config and inconsistent startup behavior |

</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: One Docracy handler layer, multiple transports later
**What:** Keep Docracy-specific request parsing, response shaping, and error mapping in one reusable MCP crate layer.
**When to use:** Immediately in phase 1, before stdio/HTTP startup exists.
**Why:** Prevents transport-specific business drift.

### Pattern 2: Library-first runtime bootstrap
**What:** Build config parsing and dependency initialization as reusable library code, not embedded directly in a binary entrypoint.
**When to use:** When the same startup path must later serve stdio and HTTP.
**Why:** Makes phase 2/3 transport addition mostly an outer wrapper problem.

### Pattern 3: Interface-local error translation
**What:** Map `CoreError`, repo setup failures, and governance/setup issues into stable MCP-facing kinds/details at the boundary.
**When to use:** Whenever the core's typed errors need to become protocol-facing errors.
**Why:** Keeps protocol concerns out of the core while preserving machine-readable detail.

### Anti-Patterns to Avoid
- **Business logic in MCP handlers:** duplicates `docracy_core` and will drift from the CLI.
- **Transport-coupled initialization:** makes stdio and HTTP diverge before both even exist.
- **Throwaway string-only error mapping:** loses structured details that automated clients need.

</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP protocol transport/handshake | Custom JSON-RPC loop | `rmcp` server support | The protocol surface has enough edge cases to justify the official SDK |
| A second command layer for Docracy operations | Bespoke MCP-only document logic | `docracy_core` service functions | Docracy already shipped the business contract |
| Per-tool startup and connection setup | Ad hoc repository/governance initialization inside handlers | Shared `crates/mcp` runtime/bootstrap helper | Repeated setup makes transport behavior inconsistent |

**Key insight:** Phase 1 succeeds by reducing new surface area, not by building more framework than Docracy needs.

</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Pulling business rules out of the core
**What goes wrong:** MCP becomes a parallel implementation of Create/Read/Query/Update.
**Why it happens:** Interface code starts “just translating” and gradually absorbs validation and policy.
**How to avoid:** Keep the MCP crate responsible for parsing, startup, and error/response mapping only.
**Warning signs:** New document/governance rules appear in `crates/mcp` without corresponding core changes.

### Pitfall 2: Mixing transport startup with reusable handler code
**What goes wrong:** stdio and HTTP end up with separate setup paths and different defaults.
**Why it happens:** Early code is written straight into `main.rs`.
**How to avoid:** Create config/runtime/bootstrap modules first and keep transport entrypoints thin.
**Warning signs:** Database/migration/governance setup gets copied into more than one place.

### Pitfall 3: Losing structured failure detail at the boundary
**What goes wrong:** Clients only see generic internal errors even when the core has better detail.
**Why it happens:** Errors are flattened to strings too early.
**How to avoid:** Preserve stable kinds and attach structured details for conflict/setup cases.
**Warning signs:** Revision conflict or validation failures lose their actionable payloads.

</common_pitfalls>

<open_questions>
## Open Questions

1. **Which exact `rmcp` API line should phase 1 target?**
   - What we know: the official repo is on the 1.x release line, but the README usage snippet still shows `rmcp = { version = "0.16.0", features = ["server"] }` and points readers to a 1.x migration guide.
   - What's unclear: the exact server-side API signatures the implementation should pin today.
   - Recommendation: verify the current crate docs/API during execution before writing the first `Cargo.toml` dependency.

2. **Does phase 1 need a binary entrypoint, or only a library crate?**
   - What we know: the roadmap requires a dedicated crate and reusable runtime boundary, while actual stdio delivery is phase 2.
   - What's unclear: whether a placeholder `docracy-mcp` binary adds value before transport work begins.
   - Recommendation: prefer a library-first crate unless the chosen `rmcp` setup makes an early binary materially simpler.

</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- `.planning/research/MCP_SERVER.md` — phase-relevant MCP architecture, transport, and client findings
- `https://github.com/modelcontextprotocol/rust-sdk` — official Rust SDK repo and current server examples
- `https://modelcontextprotocol.io/docs/sdk` — SDK overview
- `https://modelcontextprotocol.io/specification/2025-06-18/basic/transports` — stdio and Streamable HTTP transport guidance

### Codebase references
- `Cargo.toml` — current workspace members
- `crates/core/src/lib.rs` — exported core operations
- `crates/core/src/service.rs` — canonical use-case implementations
- `crates/postgres/src/lib.rs` — Postgres adapter and migrations
- `crates/cli/src/main.rs` — existing thin interface boundary pattern

</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust MCP server crate structure
- Ecosystem: `rmcp`, current workspace crates, target transports
- Patterns: thin adapter layering, runtime bootstrap, error mapping
- Pitfalls: duplicated business rules, transport-specific setup, weak error contracts

**Confidence breakdown:**
- Standard stack: HIGH - official SDK and current codebase boundary are both clear
- Architecture: HIGH - Docracy already has the right core/adapter split
- Pitfalls: HIGH - the likely failure modes are straightforward and phase-specific

**Research date:** 2026-04-06
**Valid until:** 2026-05-06 unless MCP SDK APIs change first

</metadata>

---

*Phase: 01-mcp-crate-interface-boundary*
*Research completed: 2026-04-06*
*Ready for planning: yes*
