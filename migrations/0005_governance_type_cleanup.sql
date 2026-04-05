-- Rename existing constitution rows to governance and enforce the new reserved type.

UPDATE documents
SET type = 'governance'
WHERE type = 'constitution';

DROP INDEX IF EXISTS documents_single_constitution_uq;

CREATE UNIQUE INDEX IF NOT EXISTS documents_single_governance_uq
  ON documents (type)
  WHERE type = 'governance';
