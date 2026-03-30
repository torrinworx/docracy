-- Phase 3: harden repository invariants and cursor-friendly indexes.

CREATE OR REPLACE FUNCTION document_revisions_parent_same_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  parent_document_id uuid;
BEGIN
  IF NEW.parent_revision_id IS NULL THEN
    RETURN NEW;
  END IF;

  SELECT document_id
    INTO parent_document_id
  FROM document_revisions
  WHERE id = NEW.parent_revision_id;

  IF parent_document_id IS NULL THEN
    RAISE EXCEPTION 'parent revision % does not exist', NEW.parent_revision_id
      USING ERRCODE = '23503';
  END IF;

  IF parent_document_id <> NEW.document_id THEN
    RAISE EXCEPTION 'parent revision % belongs to a different document', NEW.parent_revision_id
      USING ERRCODE = '23514';
  END IF;

  RETURN NEW;
END;
$$;

CREATE CONSTRAINT TRIGGER document_revisions_parent_same_document
AFTER INSERT OR UPDATE OF parent_revision_id, document_id ON document_revisions
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION document_revisions_parent_same_document();

CREATE OR REPLACE FUNCTION documents_current_revision_same_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  revision_document_id uuid;
BEGIN
  IF NEW.current_revision_id IS NULL THEN
    RETURN NEW;
  END IF;

  SELECT document_id
    INTO revision_document_id
  FROM document_revisions
  WHERE id = NEW.current_revision_id;

  IF revision_document_id IS NULL THEN
    RAISE EXCEPTION 'current revision % does not exist', NEW.current_revision_id
      USING ERRCODE = '23503';
  END IF;

  IF revision_document_id <> NEW.id THEN
    RAISE EXCEPTION 'current revision % belongs to a different document', NEW.current_revision_id
      USING ERRCODE = '23514';
  END IF;

  RETURN NEW;
END;
$$;

CREATE CONSTRAINT TRIGGER documents_current_revision_same_document
AFTER INSERT OR UPDATE OF current_revision_id ON documents
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION documents_current_revision_same_document();

CREATE INDEX IF NOT EXISTS documents_created_at_id_idx
  ON documents(created_at, id);

CREATE INDEX IF NOT EXISTS documents_modified_at_id_idx
  ON documents(modified_at, id);
