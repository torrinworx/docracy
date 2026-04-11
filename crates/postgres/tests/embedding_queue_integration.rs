use docracy_core::document::{DocumentStatus, DocumentType, NewDocument};
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{create_document, update_document, UpdateDocumentInput};
use docracy_postgres::PgRepository;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DOCRACY_TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
}

fn unique_schema_name() -> String {
    format!("docracy_test_{}", Uuid::new_v4().simple())
}

struct SchemaGuard {
    admin_pool: sqlx::postgres::PgPool,
    schema: String,
}

impl SchemaGuard {
    async fn create(url: &str, schema: String) -> Self {
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(url)
            .await
            .unwrap();

        let create_schema_sql = format!("CREATE SCHEMA \"{}\"", schema);
        sqlx::query(&create_schema_sql)
            .execute(&admin_pool)
            .await
            .unwrap();

        Self { admin_pool, schema }
    }
}

impl Drop for SchemaGuard {
    fn drop(&mut self) {
        let schema = self.schema.clone();
        let admin_pool = self.admin_pool.clone();
        let _ = tokio::runtime::Handle::current().block_on(async move {
            let sql = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema);
            let _ = sqlx::query(&sql).execute(&admin_pool).await;
        });
    }
}

async fn isolated_repo_scoped(url: &str, workspace_id: Option<Uuid>) -> (PgRepository, SchemaGuard) {
    let schema = unique_schema_name();
    let schema_guard = SchemaGuard::create(url, schema.clone()).await;

    let workspace_setting = workspace_id.map(|id| id.to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _meta| {
            let schema = schema.clone();
            let workspace_setting = workspace_setting.clone();
            Box::pin(async move {
                let sql = format!("SET search_path TO \"{}\", public", schema);
                sqlx::query(&sql).execute(&mut *conn).await?;
                if let Some(workspace_id) = workspace_setting {
                    sqlx::query("SELECT set_config('docracy.workspace_id', $1, false)")
                        .bind(workspace_id)
                        .execute(&mut *conn)
                        .await?;
                }
                Ok(())
            })
        })
        .connect(url)
        .await
        .unwrap();

    (PgRepository::new(pool), schema_guard)
}

#[derive(sqlx::FromRow, Debug)]
struct EmbeddingJobQueueRow {
    workspace_id: Uuid,
    document_id: Uuid,
    revision_id: Uuid,
    embed_model: String,
    source_text: String,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    attempt_count: i32,
    last_error: Option<String>,
    available_at: DateTime<Utc>,
    claimed_at: Option<DateTime<Utc>>,
}

async fn embedding_job_rows(repo: &PgRepository, document_id: Uuid) -> Vec<EmbeddingJobQueueRow> {
    sqlx::query_as::<_, EmbeddingJobQueueRow>(
        r#"
SELECT workspace_id, document_id, revision_id, embed_model, source_text,
       archived_at, deleted_at, attempt_count, last_error, available_at, claimed_at
FROM embedding_job_queue
WHERE document_id = $1
ORDER BY workspace_id ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(repo.pool())
    .await
    .unwrap()
}

#[tokio::test]
async fn embedding_queue_overwrites_pending_job_in_place() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_id = Uuid::new_v4();
    let (mut repo, _schema_guard) = isolated_repo_scoped(&url, Some(workspace_id)).await;
    repo.migrate().await.unwrap();
    repo.create_workspace(workspace_id).await.unwrap();

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let created = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"body": "alpha"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let rows = embedding_job_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].workspace_id, workspace_id);
    assert_eq!(rows[0].document_id, created.document.id.0);
    assert_eq!(rows[0].revision_id, created.revision.id.0);
    assert_eq!(rows[0].embed_model, "embeddinggemma");
    assert_eq!(rows[0].source_text, "{\"body\":\"alpha\"}");
    assert_eq!(rows[0].attempt_count, 0);
    assert!(rows[0].last_error.is_none());
    assert!(rows[0].claimed_at.is_none());
    assert!(rows[0].available_at <= Utc::now());

    let updated = update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: Some(json!({"body": "beta"})),
            extensions: None,
            status: Some(DocumentStatus::ARCHIVED.to_string()),
        },
    )
    .await
    .unwrap();

    let rows = embedding_job_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].revision_id, updated.new_revision.id.0);
    assert_eq!(rows[0].source_text, "{\"body\":\"beta\"}");
    assert!(rows[0].archived_at.is_some());
    assert!(rows[0].deleted_at.is_none());
    assert_eq!(rows[0].attempt_count, 0);
    assert!(rows[0].last_error.is_none());
}

#[tokio::test]
async fn embedding_queue_reflects_deleted_state_in_snapshot() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_id = Uuid::new_v4();
    let (mut repo, _schema_guard) = isolated_repo_scoped(&url, Some(workspace_id)).await;
    repo.migrate().await.unwrap();
    repo.create_workspace(workspace_id).await.unwrap();

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let created = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"body": "gamma"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let updated = update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: Some(json!({"body": "gamma"})),
            extensions: None,
            status: Some(DocumentStatus::DELETED.to_string()),
        },
    )
    .await
    .unwrap();

    let rows = embedding_job_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].revision_id, updated.new_revision.id.0);
    assert!(rows[0].archived_at.is_none());
    assert!(rows[0].deleted_at.is_some());
    assert_eq!(rows[0].source_text, "{\"body\":\"gamma\"}");
}
