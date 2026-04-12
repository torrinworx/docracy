# Docracy Constitution

## Purpose

Docracy is a durable memory ledger for agentic work.
It stores typed documents with immutable revision history, safe concurrency, and query primitives.

## Ownership And Immutability

1. This constitution is Docracy-owned.
2. Agents must treat it as immutable.
3. `type=governance` is reserved for Docracy. Public Create/Update must not create or mutate governance documents.
4. The system must keep exactly one stored governance document, and its content must match this constitution.

## Data Model

1. A document has: `id`, `type`, `status`, timestamps, `content` (non-null JSON), and `extensions` (JSON object).
2. `type` and `status` are compact slugs: `[a-z][a-z0-9_]*` and <= 64 characters.
3. Document history is stored as immutable revisions. The document head points at the current revision.

## Init

1. `init` must return the Docracy-owned governance bundle (markdown files).
2. `init` must return all active `type=context` documents.
3. Agents must call `init` at session start and follow governance plus active contexts.

## Tools

1. Use `query` to discover. Prefer minimal `select` first (example: `id,type,status,modified,title,summary`).
2. Use `read` only for the small set of documents that matter.
3. Use `create` to record new durable memory (non-governance types only).
4. Use `update` to change an existing document by creating a new revision.
5. Use `query_vector` to discover via semantic search.

## Updates And Concurrency

1. Every successful update must append a new revision and advance the head atomically.
2. Updates must include `expected_revision`. Stale writes must fail.

## Query

1. Guided query supports filters on: `type`, `status`, `archived`, `deleted`, `created_*`, `modified_*`.
2. Guided query defaults to `status=active` unless archived/deleted filters are explicitly requested.
3. Guided query supports keyword search over `content`.
4. Guided query must not filter on `extensions.*` (not part of the v1 contract).
5. If raw SQL query execution is supported, it must be read-only and bounded.

## Memory Discipline

1. Stored documents are for agents. Optimize for retrieval and reuse, not narration.
2. Keep information condensed to core facts and durable details. Remove filler.
3. Use list-friendly metadata for fast triage:
4. `extensions.title` is a short label.
5. `extensions.summary` is a 1 to 3 line compressed summary.
6. Put searchable substance in `content`.

## Document Type Conventions

1. Use `type` as the primary kind so guided queries stay effective.
2. Recommended durable types: `context`, `session`, `decision`, `preference`, `finding`, `task`, `playbook`, `general`.

## Token Discipline

1. Treat tokens as budget.
2. Prefer progressive disclosure: `query` -> `read few` -> act -> write condensed memory.
3. Delegate exploration and synthesis to subagents when it reduces context pollution. Subagents must return distilled results.

## Questions

1. Ask questions only when they reduce uncertainty, change the outcome, reduce risk, or unblock progress.
2. Every question must have a clear purpose. No filler.

## Context Documents

1. Context documents are mutable operating instructions (`type=context`, `status=active`).
2. Context documents must specify: tool usage patterns, note formats, note-taking triggers, delegation rules, and response style.
3. Context documents must not contradict this constitution.
