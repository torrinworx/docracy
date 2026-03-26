-- Draft Postgres schema for the document bureaucracy system described in READEME.md.
--
-- Design goals:
-- - Keep the current document state small and relational.
-- - Store revision history as immutable rows.
-- - Use JSONB for flexible content and extension fields.
-- - Support relational and keyword retrieval inside Postgres.
-- - Let an external vector store such as Qdrant mirror revision content for semantic search.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE document_status AS ENUM (
    'active',
    'archived',
    'deleted'
);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

CREATE TABLE clients (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    slug text NOT NULL UNIQUE,
    display_name text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (length(trim(slug)) > 0),
    CHECK (length(trim(display_name)) > 0)
);

CREATE TRIGGER clients_set_updated_at
BEFORE UPDATE ON clients
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE documents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id uuid NULL REFERENCES clients(id) ON DELETE SET NULL,
    kind text NOT NULL,
    status document_status NOT NULL DEFAULT 'active',
    title text NULL,
    immutable boolean NOT NULL DEFAULT false,
    system_managed boolean NOT NULL DEFAULT false,
    created_by text NULL,
    updated_by text NULL,
    current_revision_id uuid NULL,
    attributes jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    archived_at timestamptz NULL,
    deleted_at timestamptz NULL,
    CHECK (length(trim(kind)) > 0),
    CHECK (title IS NULL OR length(trim(title)) > 0),
    CHECK (archived_at IS NULL OR status = 'archived'),
    CHECK (deleted_at IS NULL OR status = 'deleted')
);

CREATE TRIGGER documents_set_updated_at
BEFORE UPDATE ON documents
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE document_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version integer NOT NULL,
    parent_revision_id uuid NULL,
    created_by text NULL,
    summary text NULL,
    content jsonb NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    -- Useful for detecting whether the external semantic index needs a refresh.
    content_hash text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    superseded_at timestamptz NULL,
    search_vector tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', COALESCE(content::text, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(metadata::text, '')), 'B')
    ) STORED,
    CONSTRAINT document_revisions_version_positive CHECK (version > 0),
    CONSTRAINT document_revisions_content_is_container CHECK (
        jsonb_typeof(content) IN ('object', 'array', 'string')
    ),
    CONSTRAINT document_revisions_unique_version UNIQUE (document_id, version),
    CONSTRAINT document_revisions_document_id_id_unique UNIQUE (document_id, id),
    CONSTRAINT document_revisions_parent_same_document_fk
        FOREIGN KEY (document_id, parent_revision_id)
        REFERENCES document_revisions(document_id, id)
        DEFERRABLE INITIALLY DEFERRED
);

ALTER TABLE documents
ADD CONSTRAINT documents_current_revision_same_document_fk
FOREIGN KEY (id, current_revision_id)
REFERENCES document_revisions(document_id, id)
DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE document_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    from_document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    to_document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    relation text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_by text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (length(trim(relation)) > 0),
    CHECK (from_document_id <> to_document_id)
);

CREATE INDEX documents_kind_status_idx
    ON documents (kind, status);

CREATE INDEX documents_client_status_idx
    ON documents (client_id, status)
    WHERE client_id IS NOT NULL;

CREATE INDEX documents_current_revision_idx
    ON documents (current_revision_id)
    WHERE current_revision_id IS NOT NULL;

CREATE INDEX documents_attributes_gin_idx
    ON documents USING gin (attributes);

CREATE INDEX document_revisions_document_version_idx
    ON document_revisions (document_id, version DESC);

CREATE INDEX document_revisions_created_at_idx
    ON document_revisions (created_at DESC);

CREATE INDEX document_revisions_content_gin_idx
    ON document_revisions USING gin (content);

CREATE INDEX document_revisions_metadata_gin_idx
    ON document_revisions USING gin (metadata);

CREATE INDEX document_revisions_search_vector_idx
    ON document_revisions USING gin (search_vector);

CREATE INDEX document_links_from_relation_idx
    ON document_links (from_document_id, relation);

CREATE INDEX document_links_to_relation_idx
    ON document_links (to_document_id, relation);

CREATE VIEW current_document_state AS
SELECT
    d.id,
    d.client_id,
    d.kind,
    d.status,
    d.title,
    d.immutable,
    d.system_managed,
    d.created_by,
    d.updated_by,
    d.attributes,
    d.created_at,
    d.updated_at,
    d.archived_at,
    d.deleted_at,
    r.id AS revision_id,
    r.version,
    r.parent_revision_id,
    r.summary,
    r.content,
    r.metadata AS revision_metadata,
    r.content_hash,
    r.created_by AS revision_created_by,
    r.created_at AS revision_created_at,
    r.superseded_at
FROM documents d
LEFT JOIN document_revisions r ON r.id = d.current_revision_id;

-- Suggested Qdrant mirroring model:
-- - Use document_revisions.id as the external point id.
-- - Mirror the latest searchable revision content and metadata into Qdrant.
-- - Use content_hash to detect whether a revision needs re-embedding.
-- - On delete/archive, update Qdrant payload or remove the point based on retrieval policy.

-- Suggested update flow for the Rust repository layer:
-- 1. BEGIN;
-- 2. SELECT * FROM documents WHERE id = $1 FOR UPDATE;
-- 3. INSERT INTO document_revisions (..., version = previous_version + 1, ...);
-- 4. UPDATE documents SET current_revision_id = $new_revision_id, updated_by = $actor WHERE id = $1;
-- 5. COMMIT;
