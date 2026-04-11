-- Phase 13: retryable embedding job queue for async Ollama workers.

CREATE TABLE IF NOT EXISTS embedding_job_queue (
  workspace_id uuid NOT NULL DEFAULT coalesce(
    current_setting('docracy.workspace_id', true)::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
  ),
  document_id uuid NOT NULL,
  revision_id uuid NOT NULL,
  embed_model text NOT NULL,
  source_text text NOT NULL,

  archived_at timestamptz,
  deleted_at timestamptz,

  attempt_count integer NOT NULL DEFAULT 0,
  last_error text,
  available_at timestamptz NOT NULL DEFAULT now(),
  claimed_at timestamptz,

  created_at timestamptz NOT NULL DEFAULT now(),
  modified_at timestamptz NOT NULL DEFAULT now(),

  CONSTRAINT embedding_job_queue_document_fk
    FOREIGN KEY (workspace_id, document_id)
    REFERENCES documents (workspace_id, id)
    ON DELETE CASCADE,
  CONSTRAINT embedding_job_queue_revision_fk
    FOREIGN KEY (workspace_id, revision_id)
    REFERENCES document_revisions (workspace_id, id)
    ON DELETE CASCADE,
  CONSTRAINT embedding_job_queue_workspace_fk
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
  CONSTRAINT embedding_job_queue_pk PRIMARY KEY (workspace_id, document_id, embed_model)
);

CREATE INDEX IF NOT EXISTS embedding_job_queue_workspace_available_at_modified_at_idx
  ON embedding_job_queue (workspace_id, available_at ASC, modified_at DESC, document_id ASC);

ALTER TABLE embedding_job_queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE embedding_job_queue FORCE ROW LEVEL SECURITY;

CREATE POLICY embedding_job_queue_workspace_isolation ON embedding_job_queue
  USING (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  )
  WITH CHECK (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  );
