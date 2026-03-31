# Docracy Constitution

## Purpose

Docracy is a durable document store for agentic work. It exists to keep project knowledge as typed, versioned documents with reliable read, query, and update behavior.

## Constitutional Rules

1. This constitution is repo-owned and immutable through normal user-facing document operations.
2. The system must keep exactly one stored constitution document, and its content must match this file.
3. `init` must load the governance bundle and the active `context` documents.
4. Documents are identified by stable document IDs and tracked through immutable revisions.
5. Every successful update must append a new revision and advance the document head atomically.
6. Updates must include the caller's expected current revision. Stale writes must fail rather than overwrite newer work.
7. The `constitution` document type is reserved for the system. Public create and update flows must not allow agents to create or mutate constitution documents.
8. Documents may carry `type`, `status`, timestamps, `content`, and `extensions`. `content` and `extensions` are agent-defined JSON values unless governance says otherwise.
9. Query behavior applies to current documents. v1 supports structured filtering, ordering, pagination, and keyword search over document content.
10. Extension data may be stored and returned, but querying or filtering on `extensions.*` is not part of the v1 contract.
11. `context` documents are mutable governance guidance. They may refine conventions for agents, but they must not contradict this constitution.

## Change Policy

Changes to this constitution require editing this repo file. Interface-specific behavior may exist, but it must preserve the rules above.
