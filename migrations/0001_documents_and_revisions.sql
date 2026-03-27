-- Phase 1: canonical schema for documents + revision history.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS documents (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  type text NOT NULL,
  status text NOT NULL,

  created_at timestamptz NOT NULL DEFAULT now(),
  modified_at timestamptz NOT NULL DEFAULT now(),

  current_revision_id uuid,
  archived_at timestamptz,
  deleted_at timestamptz,

  content jsonb NOT NULL,
  extensions jsonb NOT NULL DEFAULT '{}'::jsonb,

  CONSTRAINT documents_extensions_object CHECK (jsonb_typeof(extensions) = 'object'),
  CONSTRAINT documents_modified_gte_created CHECK (modified_at >= created_at),
  CONSTRAINT documents_archived_requires_ts CHECK ((status = 'archived') = (archived_at IS NOT NULL)),
  CONSTRAINT documents_deleted_requires_ts CHECK ((status = 'deleted') = (deleted_at IS NOT NULL)),
  CONSTRAINT documents_not_archived_and_deleted CHECK (NOT (archived_at IS NOT NULL AND deleted_at IS NOT NULL))
);

CREATE TABLE IF NOT EXISTS document_revisions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  document_id uuid NOT NULL REFERENCES documents(id),
  version integer NOT NULL,
  parent_revision_id uuid REFERENCES document_revisions(id),

  created_at timestamptz NOT NULL DEFAULT now(),
  superseded_at timestamptz,

  content jsonb NOT NULL,
  extensions jsonb NOT NULL DEFAULT '{}'::jsonb,

  CONSTRAINT document_revisions_version_positive CHECK (version > 0),
  CONSTRAINT document_revisions_extensions_object CHECK (jsonb_typeof(extensions) = 'object'),
  CONSTRAINT document_revisions_parent_not_self CHECK (parent_revision_id IS NULL OR parent_revision_id <> id)
);

ALTER TABLE documents
  ADD CONSTRAINT documents_current_revision_fk
  FOREIGN KEY (current_revision_id) REFERENCES document_revisions(id)
  DEFERRABLE INITIALLY DEFERRED;

CREATE UNIQUE INDEX IF NOT EXISTS document_revisions_document_id_version_uq
  ON document_revisions(document_id, version);

CREATE INDEX IF NOT EXISTS documents_type_idx ON documents(type);
CREATE INDEX IF NOT EXISTS documents_status_idx ON documents(status);
CREATE INDEX IF NOT EXISTS documents_modified_at_idx ON documents(modified_at);
CREATE INDEX IF NOT EXISTS document_revisions_document_id_idx ON document_revisions(document_id);

CREATE INDEX IF NOT EXISTS documents_extensions_gin_idx
  ON documents USING GIN (extensions);
CREATE INDEX IF NOT EXISTS document_revisions_extensions_gin_idx
  ON document_revisions USING GIN (extensions);
