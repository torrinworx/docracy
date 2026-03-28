-- Full-text search support for Query().

ALTER TABLE documents
  ADD COLUMN IF NOT EXISTS content_search_tsv tsvector
  GENERATED ALWAYS AS (
    to_tsvector('english', coalesce(content::text, ''))
  ) STORED;

CREATE INDEX IF NOT EXISTS documents_content_search_tsv_gin_idx
  ON documents USING GIN (content_search_tsv);
