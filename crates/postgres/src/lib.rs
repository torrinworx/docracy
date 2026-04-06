#![forbid(unsafe_code)]

use async_trait::async_trait;
use docracy_core::document::{Document, DocumentStatus, DocumentType};
use docracy_core::errors::RepoError;
use docracy_core::ids::{DocumentId, RevisionId};
use docracy_core::query::{
    DocumentQuery, DocumentQueryCursor, DocumentQueryOrder, DocumentQueryResult,
};
use docracy_core::repository::Repository;
use docracy_core::revision::DocumentRevision;
use serde_json::{Map, Value};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;

const RAW_QUERY_LIMIT_CEILING: u32 = 100;
const RAW_QUERY_DEFAULT_LIMIT: u32 = 10;
const RAW_QUERY_TIMEOUT_CEILING_MS: u64 = 5000;

pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new().connect(database_url).await?;
        Ok(Self::new(pool))
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("../../migrations").run(&self.pool).await
    }
}

#[derive(sqlx::FromRow)]
struct DocumentRow {
    id: Uuid,
    #[sqlx(rename = "type")]
    doc_type: String,
    status: String,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    current_revision_id: Option<Uuid>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    content: Value,
    extensions: Value,
}

#[derive(sqlx::FromRow)]
struct RevisionRow {
    id: Uuid,
    document_id: Uuid,
    version: i32,
    parent_revision_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    superseded_at: Option<DateTime<Utc>>,
    content: Value,
    extensions: Value,
}

fn map_sqlx_error(e: sqlx::Error) -> RepoError {
    match &e {
        sqlx::Error::Database(db) => {
            if db.code().as_deref() == Some("23505") {
                return RepoError::Conflict;
            }
            RepoError::Storage(format!("{db}"))
        }
        _ => RepoError::Storage(e.to_string()),
    }
}

fn value_to_object_map(v: Value) -> Result<Map<String, Value>, RepoError> {
    match v {
        Value::Object(m) => Ok(m),
        _ => Err(RepoError::Storage(
            "extensions must be a JSON object".to_string(),
        )),
    }
}

fn doc_row_to_core(row: DocumentRow) -> Result<Document, RepoError> {
    let doc_type =
        DocumentType::new(row.doc_type).map_err(|e| RepoError::Storage(e.to_string()))?;
    let status = DocumentStatus::new(row.status).map_err(|e| RepoError::Storage(e.to_string()))?;
    let extensions = value_to_object_map(row.extensions)?;
    Ok(Document {
        id: DocumentId(row.id),
        doc_type,
        status,
        created_at: row.created_at,
        modified_at: row.modified_at,
        current_revision_id: row.current_revision_id.map(RevisionId),
        archived_at: row.archived_at,
        deleted_at: row.deleted_at,
        content: row.content,
        extensions,
    })
}

fn rev_row_to_core(row: RevisionRow) -> Result<DocumentRevision, RepoError> {
    let extensions = value_to_object_map(row.extensions)?;
    let version: u32 = row
        .version
        .try_into()
        .map_err(|_| RepoError::Storage("revision version out of range".to_string()))?;
    Ok(DocumentRevision {
        id: RevisionId(row.id),
        document_id: DocumentId(row.document_id),
        version,
        parent_revision_id: row.parent_revision_id.map(RevisionId),
        created_at: row.created_at,
        superseded_at: row.superseded_at,
        content: row.content,
        extensions,
    })
}

fn extensions_as_value(ext: &Map<String, Value>) -> Value {
    Value::Object(ext.clone())
}

fn version_to_i32(version: u32) -> Result<i32, RepoError> {
    i32::try_from(version)
        .map_err(|_| RepoError::Storage("revision version out of range".to_string()))
}

fn raw_query_limit(limit: Option<u32>) -> i64 {
    i64::from(limit.unwrap_or(RAW_QUERY_DEFAULT_LIMIT).clamp(1, RAW_QUERY_LIMIT_CEILING))
}

fn raw_query_timeout(timeout_ms: Option<u64>) -> u64 {
    timeout_ms
        .unwrap_or(RAW_QUERY_TIMEOUT_CEILING_MS)
        .clamp(1, RAW_QUERY_TIMEOUT_CEILING_MS)
}

fn wrap_raw_query(sql: &str) -> String {
    format!("SELECT to_jsonb(raw_query) AS row FROM ({sql}) AS raw_query")
}

fn wrap_raw_count_query(sql: &str) -> String {
    format!("SELECT COUNT(*)::bigint FROM ({sql}) AS raw_query")
}

#[async_trait(?Send)]
impl Repository for PgRepository {
    async fn create_document_with_revision(
        &mut self,
        doc: Document,
        rev: DocumentRevision,
    ) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        sqlx::query("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        sqlx::query(
            r#"
INSERT INTO documents (
  id, "type", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions
)
VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(doc.id.0)
        .bind(doc.doc_type.as_str())
        .bind(doc.status.as_str())
        .bind(doc.created_at)
        .bind(doc.modified_at)
        .bind(doc.current_revision_id.map(|r| r.0))
        .bind(doc.archived_at)
        .bind(doc.deleted_at)
        .bind(doc.content)
        .bind(extensions_as_value(&doc.extensions))
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query(
            r#"
INSERT INTO document_revisions (
  id, document_id, version, parent_revision_id, created_at, superseded_at, content, extensions
)
VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(rev.id.0)
        .bind(rev.document_id.0)
        .bind(version_to_i32(rev.version)?)
        .bind(rev.parent_revision_id.map(|r| r.0))
        .bind(rev.created_at)
        .bind(rev.superseded_at)
        .bind(rev.content)
        .bind(extensions_as_value(&rev.extensions))
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn update_document_with_revisions(
        &mut self,
        doc: Document,
        expected_head: RevisionId,
        superseded: DocumentRevision,
        new_rev: DocumentRevision,
    ) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        sqlx::query("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        let current_head = sqlx::query_scalar::<_, Option<Uuid>>(
            r#"
SELECT current_revision_id
FROM documents
WHERE id = $1
FOR UPDATE
            "#,
        )
        .bind(doc.id.0)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(current_head) = current_head else {
            return Err(RepoError::Storage("update of missing document".to_string()));
        };
        let Some(current_head) = current_head else {
            return Err(RepoError::Storage(
                "document missing current revision".to_string(),
            ));
        };

        if current_head != expected_head.0 {
            return Err(RepoError::Conflict);
        }

        let updated = sqlx::query(
            r#"
UPDATE document_revisions
SET superseded_at = $2
WHERE id = $1
            "#,
        )
        .bind(superseded.id.0)
        .bind(superseded.superseded_at)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        if updated != 1 {
            return Err(RepoError::Storage(
                "update of missing superseded revision".to_string(),
            ));
        }

        sqlx::query(
            r#"
INSERT INTO document_revisions (
  id, document_id, version, parent_revision_id, created_at, superseded_at, content, extensions
)
VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(new_rev.id.0)
        .bind(new_rev.document_id.0)
        .bind(version_to_i32(new_rev.version)?)
        .bind(new_rev.parent_revision_id.map(|r| r.0))
        .bind(new_rev.created_at)
        .bind(new_rev.superseded_at)
        .bind(new_rev.content)
        .bind(extensions_as_value(&new_rev.extensions))
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let updated = sqlx::query(
            r#"
UPDATE documents
SET
  "type" = $2,
  status = $3,
  created_at = $4,
  modified_at = $5,
  current_revision_id = $6,
  archived_at = $7,
  deleted_at = $8,
  content = $9,
  extensions = $10
WHERE id = $1
            "#,
        )
        .bind(doc.id.0)
        .bind(doc.doc_type.as_str())
        .bind(doc.status.as_str())
        .bind(doc.created_at)
        .bind(doc.modified_at)
        .bind(doc.current_revision_id.map(|r| r.0))
        .bind(doc.archived_at)
        .bind(doc.deleted_at)
        .bind(doc.content)
        .bind(extensions_as_value(&doc.extensions))
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        if updated != 1 {
            return Err(RepoError::Storage("update of missing document".to_string()));
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn update_document(&mut self, doc: Document) -> Result<(), RepoError> {
        let updated = sqlx::query(
            r#"
UPDATE documents
SET
  "type" = $2,
  status = $3,
  created_at = $4,
  modified_at = $5,
  current_revision_id = $6,
  archived_at = $7,
  deleted_at = $8,
  content = $9,
  extensions = $10
WHERE id = $1
            "#,
        )
        .bind(doc.id.0)
        .bind(doc.doc_type.as_str())
        .bind(doc.status.as_str())
        .bind(doc.created_at)
        .bind(doc.modified_at)
        .bind(doc.current_revision_id.map(|r| r.0))
        .bind(doc.archived_at)
        .bind(doc.deleted_at)
        .bind(doc.content)
        .bind(extensions_as_value(&doc.extensions))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        if updated != 1 {
            return Err(RepoError::Storage("update of missing document".to_string()));
        }
        Ok(())
    }

    async fn get_document(&self, id: DocumentId) -> Result<Option<Document>, RepoError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
SELECT id, "type", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions
FROM documents
WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(doc_row_to_core).transpose()
    }

    async fn get_documents(&self, ids: &[DocumentId]) -> Result<Vec<Document>, RepoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as::<_, DocumentRow>(
            r#"
SELECT id, "type", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions
FROM documents
WHERE id = ANY($1)
            "#,
        )
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(doc_row_to_core).collect()
    }

    async fn find_latest_document_by_type(
        &self,
        doc_type: &str,
    ) -> Result<Option<Document>, RepoError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
SELECT id, "type", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions
FROM documents
WHERE "type" = $1
ORDER BY modified_at DESC
LIMIT 1
            "#,
        )
        .bind(doc_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        row.map(doc_row_to_core).transpose()
    }

    async fn insert_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        sqlx::query(
            r#"
INSERT INTO document_revisions (
  id, document_id, version, parent_revision_id, created_at, superseded_at, content, extensions
)
VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(rev.id.0)
        .bind(rev.document_id.0)
        .bind(version_to_i32(rev.version)?)
        .bind(rev.parent_revision_id.map(|r| r.0))
        .bind(rev.created_at)
        .bind(rev.superseded_at)
        .bind(rev.content)
        .bind(extensions_as_value(&rev.extensions))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn update_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        let updated = sqlx::query(
            r#"
UPDATE document_revisions
SET
  document_id = $2,
  version = $3,
  parent_revision_id = $4,
  created_at = $5,
  superseded_at = $6,
  content = $7,
  extensions = $8
WHERE id = $1
            "#,
        )
        .bind(rev.id.0)
        .bind(rev.document_id.0)
        .bind(version_to_i32(rev.version)?)
        .bind(rev.parent_revision_id.map(|r| r.0))
        .bind(rev.created_at)
        .bind(rev.superseded_at)
        .bind(rev.content)
        .bind(extensions_as_value(&rev.extensions))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        if updated != 1 {
            return Err(RepoError::Storage("update of missing revision".to_string()));
        }
        Ok(())
    }

    async fn get_revision(&self, id: RevisionId) -> Result<Option<DocumentRevision>, RepoError> {
        let row = sqlx::query_as::<_, RevisionRow>(
            r#"
SELECT id, document_id, version, parent_revision_id, created_at, superseded_at, content, extensions
FROM document_revisions
WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        row.map(rev_row_to_core).transpose()
    }

    async fn list_active_context_documents(&self) -> Result<Vec<Document>, RepoError> {
        let rows = sqlx::query_as::<_, DocumentRow>(
            r#"
SELECT id, "type", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions
FROM documents
WHERE "type" = $1 AND status = $2
ORDER BY modified_at DESC
            "#,
        )
        .bind(DocumentType::CONTEXT)
        .bind(DocumentStatus::ACTIVE)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(doc_row_to_core).collect()
    }

    async fn query_documents(
        &self,
        query: DocumentQuery,
    ) -> Result<DocumentQueryResult, RepoError> {
        use sqlx::Postgres;

        // Count (ignores cursor): total matching rows.
        let mut count_qb =
            sqlx::QueryBuilder::<Postgres>::new("SELECT COUNT(*)::bigint FROM documents WHERE 1=1");
        push_query_filters(&mut count_qb, &query, false);
        let total_count: i64 = count_qb
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        // Page.
        let mut qb = sqlx::QueryBuilder::<Postgres>::new(
            "SELECT id, \"type\", status, created_at, modified_at, current_revision_id, archived_at, deleted_at, content, extensions FROM documents WHERE 1=1",
        );
        push_query_filters(&mut qb, &query, true);

        match query.order {
            DocumentQueryOrder::ModifiedDesc => qb.push(" ORDER BY modified_at DESC, id DESC"),
            DocumentQueryOrder::ModifiedAsc => qb.push(" ORDER BY modified_at ASC, id ASC"),
            DocumentQueryOrder::CreatedDesc => qb.push(" ORDER BY created_at DESC, id DESC"),
            DocumentQueryOrder::CreatedAsc => qb.push(" ORDER BY created_at ASC, id ASC"),
        };

        // Fetch one extra row to know if there's another page.
        qb.push(" LIMIT ")
            .push_bind(i64::from(query.limit.saturating_add(1)));

        let rows: Vec<DocumentRow> = qb
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        let mut docs: Vec<Document> = rows
            .into_iter()
            .map(doc_row_to_core)
            .collect::<Result<_, _>>()?;

        let has_more = docs.len() > query.limit as usize;
        docs.truncate(query.limit as usize);

        let next_cursor = if has_more {
            docs.last().map(|d| {
                let ts = match query.order {
                    DocumentQueryOrder::ModifiedDesc | DocumentQueryOrder::ModifiedAsc => {
                        d.modified_at
                    }
                    DocumentQueryOrder::CreatedDesc | DocumentQueryOrder::CreatedAsc => {
                        d.created_at
                    }
                };
                DocumentQueryCursor { ts, id: d.id }
            })
        } else {
            None
        };

        Ok(DocumentQueryResult {
            documents: docs,
            total_count: total_count.max(0) as u64,
            next_cursor,
        })
    }

    async fn query_raw_documents(
        &self,
        query: docracy_core::query::RawQueryInput,
    ) -> Result<docracy_core::query::RawQueryResult, RepoError> {
        let sql = query.sql.trim();
        if sql.is_empty() {
            return Err(RepoError::Storage(
                "raw SQL query must not be empty".to_string(),
            ));
        }

        let limit = raw_query_limit(query.limit);
        let timeout_ms = raw_query_timeout(query.timeout_ms);
        let count_sql = wrap_raw_count_query(sql);
        let page_sql = format!("{} LIMIT $1", wrap_raw_query(sql));

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        sqlx::query_scalar::<_, String>("SELECT set_config('statement_timeout', $1, true)")
            .bind(format!("{timeout_ms}ms"))
            .fetch_one(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        let total_count: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        let rows: Vec<Value> = sqlx::query_scalar(&page_sql)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        let rows = rows
            .into_iter()
            .map(|row| match row {
                Value::Object(row) => row,
                other => {
                    let mut map = Map::new();
                    map.insert("row".to_string(), other);
                    map
                }
            })
            .collect();

        Ok(docracy_core::query::RawQueryResult {
            rows,
            total_count: total_count.max(0) as u64,
        })
    }
}

fn push_query_filters<'a>(
    qb: &mut sqlx::QueryBuilder<'a, sqlx::Postgres>,
    query: &'a DocumentQuery,
    include_cursor: bool,
) {
    if let Some(types) = &query.types {
        qb.push(" AND \"type\" = ANY(").push_bind(types).push(")");
    }
    if let Some(statuses) = &query.statuses {
        qb.push(" AND status = ANY(").push_bind(statuses).push(")");
    }
    if let Some(archived) = query.archived {
        if archived {
            qb.push(" AND archived_at IS NOT NULL");
        } else {
            qb.push(" AND archived_at IS NULL");
        }
    }
    if let Some(deleted) = query.deleted {
        if deleted {
            qb.push(" AND deleted_at IS NOT NULL");
        } else {
            qb.push(" AND deleted_at IS NULL");
        }
    }
    if let Some(gte) = query.created_gte {
        qb.push(" AND created_at >= ").push_bind(gte);
    }
    if let Some(lte) = query.created_lte {
        qb.push(" AND created_at <= ").push_bind(lte);
    }
    if let Some(gte) = query.modified_gte {
        qb.push(" AND modified_at >= ").push_bind(gte);
    }
    if let Some(lte) = query.modified_lte {
        qb.push(" AND modified_at <= ").push_bind(lte);
    }

    if let Some(q) = &query.query {
        qb.push(" AND content_search_tsv @@ websearch_to_tsquery('english', ")
            .push_bind(q)
            .push(")");
    }

    if include_cursor {
        if let Some(c) = &query.cursor {
            match query.order {
                DocumentQueryOrder::ModifiedDesc => {
                    qb.push(" AND (modified_at, id) < (")
                        .push_bind(c.ts)
                        .push(", ")
                        .push_bind(c.id.0)
                        .push(")");
                }
                DocumentQueryOrder::ModifiedAsc => {
                    qb.push(" AND (modified_at, id) > (")
                        .push_bind(c.ts)
                        .push(", ")
                        .push_bind(c.id.0)
                        .push(")");
                }
                DocumentQueryOrder::CreatedDesc => {
                    qb.push(" AND (created_at, id) < (")
                        .push_bind(c.ts)
                        .push(", ")
                        .push_bind(c.id.0)
                        .push(")");
                }
                DocumentQueryOrder::CreatedAsc => {
                    qb.push(" AND (created_at, id) > (")
                        .push_bind(c.ts)
                        .push(", ")
                        .push_bind(c.id.0)
                        .push(")");
                }
            }
        }
    }
}
