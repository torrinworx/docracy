-- Phase 12: current-state vector mirror queue for derived embeddings.

CREATE TABLE IF NOT EXISTS vector_mirror_queue (
  workspace_id uuid NOT NULL DEFAULT coalesce(
    current_setting('docracy.workspace_id', true)::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
  ),
  document_id uuid NOT NULL,
  revision_id uuid NOT NULL,

  archived_at timestamptz,
  deleted_at timestamptz,

  embedding_dimension integer NOT NULL,
  embedding jsonb NOT NULL,

  created_at timestamptz NOT NULL DEFAULT now(),
  modified_at timestamptz NOT NULL DEFAULT now(),

  CONSTRAINT vector_mirror_queue_embedding_is_array CHECK (jsonb_typeof(embedding) = 'array'),
  CONSTRAINT vector_mirror_queue_embedding_dimension_positive CHECK (embedding_dimension > 0),
  CONSTRAINT vector_mirror_queue_not_archived_and_deleted CHECK (
    NOT (archived_at IS NOT NULL AND deleted_at IS NOT NULL)
  ),
  CONSTRAINT vector_mirror_queue_document_fk
    FOREIGN KEY (workspace_id, document_id)
    REFERENCES documents (workspace_id, id)
    ON DELETE CASCADE,
  CONSTRAINT vector_mirror_queue_revision_fk
    FOREIGN KEY (workspace_id, revision_id)
    REFERENCES document_revisions (workspace_id, id)
    ON DELETE CASCADE,
  CONSTRAINT vector_mirror_queue_workspace_fk
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
  CONSTRAINT vector_mirror_queue_pk PRIMARY KEY (workspace_id, document_id)
);

CREATE INDEX IF NOT EXISTS vector_mirror_queue_workspace_modified_at_id_idx
  ON vector_mirror_queue (workspace_id, modified_at DESC, document_id DESC);

ALTER TABLE vector_mirror_queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE vector_mirror_queue FORCE ROW LEVEL SECURITY;

CREATE POLICY vector_mirror_queue_workspace_isolation ON vector_mirror_queue
  USING (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  )
  WITH CHECK (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  );
