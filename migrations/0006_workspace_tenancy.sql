-- Phase 8: workspace tenancy with RLS isolation.

CREATE TABLE IF NOT EXISTS workspaces (
  id uuid PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO workspaces (id)
VALUES ('00000000-0000-0000-0000-000000000000'::uuid)
ON CONFLICT (id) DO NOTHING;

ALTER TABLE documents
  ADD COLUMN IF NOT EXISTS workspace_id uuid;

ALTER TABLE document_revisions
  ADD COLUMN IF NOT EXISTS workspace_id uuid;

UPDATE documents
SET workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
WHERE workspace_id IS NULL;

UPDATE document_revisions
SET workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
WHERE workspace_id IS NULL;

ALTER TABLE documents
  ALTER COLUMN workspace_id SET DEFAULT coalesce(
    current_setting('docracy.workspace_id', true)::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
  );

ALTER TABLE document_revisions
  ALTER COLUMN workspace_id SET DEFAULT coalesce(
    current_setting('docracy.workspace_id', true)::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
  );

ALTER TABLE documents
  ALTER COLUMN workspace_id SET NOT NULL;

ALTER TABLE document_revisions
  ALTER COLUMN workspace_id SET NOT NULL;

ALTER TABLE documents
  ADD CONSTRAINT documents_workspace_id_id_uq UNIQUE (workspace_id, id);

ALTER TABLE document_revisions
  ADD CONSTRAINT document_revisions_workspace_id_id_uq UNIQUE (workspace_id, id);

ALTER TABLE documents
  ADD CONSTRAINT documents_workspace_id_fkey
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id);

ALTER TABLE document_revisions
  ADD CONSTRAINT document_revisions_workspace_id_fkey
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id);

ALTER TABLE document_revisions
  ADD CONSTRAINT document_revisions_workspace_document_fkey
  FOREIGN KEY (workspace_id, document_id)
  REFERENCES documents (workspace_id, id)
  DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE document_revisions
  ADD CONSTRAINT document_revisions_workspace_parent_fkey
  FOREIGN KEY (workspace_id, parent_revision_id)
  REFERENCES document_revisions (workspace_id, id)
  DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE documents
  ADD CONSTRAINT documents_workspace_current_revision_fkey
  FOREIGN KEY (workspace_id, current_revision_id)
  REFERENCES document_revisions (workspace_id, id)
  DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX IF NOT EXISTS documents_workspace_created_at_id_idx
  ON documents (workspace_id, created_at, id);

CREATE INDEX IF NOT EXISTS documents_workspace_modified_at_id_idx
  ON documents (workspace_id, modified_at, id);

CREATE INDEX IF NOT EXISTS documents_workspace_type_modified_at_id_idx
  ON documents (workspace_id, "type", modified_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS documents_workspace_status_modified_at_id_idx
  ON documents (workspace_id, status, modified_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS document_revisions_workspace_document_id_idx
  ON document_revisions (workspace_id, document_id);

CREATE INDEX IF NOT EXISTS document_revisions_workspace_parent_revision_id_idx
  ON document_revisions (workspace_id, parent_revision_id);

ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents FORCE ROW LEVEL SECURITY;

ALTER TABLE document_revisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_revisions FORCE ROW LEVEL SECURITY;

CREATE POLICY documents_workspace_isolation ON documents
  USING (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  )
  WITH CHECK (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  );

CREATE POLICY document_revisions_workspace_isolation ON document_revisions
  USING (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  )
  WITH CHECK (
    workspace_id = '00000000-0000-0000-0000-000000000000'::uuid
    OR workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
  );
