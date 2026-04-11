---
phase: 13-async-ollama-embedding-worker-and-qdrant-only-vector-index
status: complete
date: 2026-04-10
sources:
  - .planning/PROJECT.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
  - .planning/research/ARCHITECTURE.md
  - .planning/research/PITFALLS.md
  - .planning/research/SUMMARY.md
  - .planning/phases/12-vector-mirror-helper-and-vector-query-support/12-01-SUMMARY.md
  - .planning/phases/12-vector-mirror-helper-and-vector-query-support/12-02-SUMMARY.md
  - compose.yml
  - .env.example
---

# Phase 13 Research: Async Ollama Embedding Worker and Qdrant-Only Vector Index

## What this phase should do

Move embedding generation out of the document write path and into a workspace-scoped async worker that calls Ollama, then upserts Qdrant as the only derived vector index. Postgres stays canonical for document state, revision history, and retryable job tracking.

## Recommended design

1. **Keep the canonical document store unchanged.**
   - Document CRUD still happens in `docracy-core` + `docracy-postgres`.
   - The new queue row stores a text snapshot derived from the current document revision, not a vector blob.

2. **Use a Postgres queue with at-least-once semantics.**
   - Enqueue/overwrite a single pending row per `(workspace_id, document_id, embed_model)`.
   - Claim rows with `FOR UPDATE SKIP LOCKED`.
   - On success, delete the row.
   - On Ollama/Qdrant failure, keep the row retryable with attempt/error metadata.

3. **Bind the worker to a workspace.**
   - Use `WORKSPACE_ID` for the worker process, matching the existing workspace-scoped database model.
   - This keeps RLS and collection naming aligned with the current architecture.

4. **Use Ollama’s embed endpoint directly.**
   - Default local endpoint: `http://127.0.0.1:11434`.
   - Default local model: `embeddinggemma`.
   - Request shape: `POST /api/embed` with `{ model, input }`.

5. **Keep Qdrant as the derived index only.**
   - Continue workspace-scoped collection naming (`docracy_workspace_{workspace_id}`).
   - Upsert points keyed by `document_id` and include payload metadata for workspace/revision/model/archive state.

6. **Serialize document content canonically before embedding.**
   - Early support can embed JSON content via stable JSON stringification.
   - This matches the repo’s current document shape and avoids introducing a second content model in phase 13.

## Why this direction

- It matches the existing architectural rule: canonical documents in Postgres, derived retrieval in Qdrant.
- It removes synchronous embedding work from document writes.
- It gives the worker a clean retry surface without changing the public Init/Create/Read/Query/Update contract.
- It keeps the local dev story simple: compose already runs Ollama and Qdrant; the worker can be started with the repo workspace env.

## Guardrails

- Do not move business rules into the worker; it should only claim jobs, embed text, and write derived vectors.
- Do not store raw vectors in Postgres for this phase.
- Do not expand the query surface in this phase; phase 12 already covered embedding-based query routing.

## Sources

- Ollama embed API: `/api/embed`, returns `embeddings: number[][]`.
- Qdrant collections: vectors in a collection must share the same dimensionality; cosine distance is the current fit.
- Existing phase 12 vector mirror/query summaries.
- Existing research notes on async indexers and the pitfall of storing embeddings without revision/model linkage.
