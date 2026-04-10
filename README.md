# Docracy

Docracy lets agents store durable context in Postgres instead of a filesystem.

## Async embedding worker

The `docracy-indexer` worker drains workspace-scoped embedding jobs, sends text to Ollama, and upserts vectors into Qdrant.

- Postgres remains canonical for documents, revisions, and retry metadata.
- Qdrant is the derived index only.
- The worker is bound to one `WORKSPACE_ID` at startup.

### Local run

1. Provision or reuse a workspace ID.
2. Pull the embedding model:
   `ollama pull embeddinggemma`
3. Start Postgres, Ollama, and Qdrant.
4. Set env vars from `.env.example`.
5. Run the worker:
   `cargo run -p docracy-postgres --bin docracy-indexer`

### Worker env

- `WORKSPACE_ID`
- `DATABASE_URL`
- `OLLAMA_URL`
- `OLLAMA_EMBED_MODEL`
- `QDRANT_URL`
- `POLL_INTERVAL_MS`
- `BATCH_SIZE`
