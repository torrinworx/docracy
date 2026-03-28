-- Enforce exactly one constitution document ever.

CREATE UNIQUE INDEX IF NOT EXISTS documents_single_constitution_uq
  ON documents (type)
  WHERE type = 'constitution';
