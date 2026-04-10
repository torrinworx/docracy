use docracy_core::document::{DocumentStatus, DocumentType, NewDocument};
use docracy_core::query::RawQueryInput;
use docracy_core::repository::Repository;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{
    create_document, init_bundle, query_documents, update_document, FsGovernanceSource, QueryInput,
    UpdateDocumentInput,
};
use docracy_postgres::PgRepository;
use serde_json::json;
use serde_json::Map;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::{DateTime, Utc};
use tempfile::TempDir;
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

async fn isolated_repo(url: &str) -> (PgRepository, SchemaGuard) {
    isolated_repo_scoped(url, None).await
}

async fn isolated_repo_scoped(
    url: &str,
    workspace_id: Option<Uuid>,
) -> (PgRepository, SchemaGuard) {
    let schema = unique_schema_name();
    let schema_guard = SchemaGuard::create(url, schema.clone()).await;
    let repo = repo_on_schema(url, schema, workspace_id).await;

    (repo, schema_guard)
}

async fn repo_on_schema(url: &str, schema: String, workspace_id: Option<Uuid>) -> PgRepository {
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

    PgRepository::new(pool)
}

async fn assert_index_exists(repo: &PgRepository, index_name: &str) {
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(index_name)
        .fetch_one(repo.pool())
        .await
        .unwrap();

    assert_eq!(exists.as_deref(), Some(index_name));
}

async fn assert_index_absent(repo: &PgRepository, index_name: &str) {
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(index_name)
        .fetch_one(repo.pool())
        .await
        .unwrap();

    assert_eq!(exists, None);
}

async fn workspace_id_for_document(repo: &PgRepository, id: docracy_core::DocumentId) -> Uuid {
    sqlx::query_scalar("SELECT workspace_id FROM documents WHERE id = $1")
        .bind(id.0)
        .fetch_one(repo.pool())
        .await
        .unwrap()
}

async fn workspace_exists(repo: &PgRepository, workspace_id: Uuid) -> bool {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .fetch_optional(repo.pool())
        .await
        .unwrap()
        .is_some()
}

async fn seed_documents(repo: &PgRepository, count: i32) {
    sqlx::query(
        r#"
INSERT INTO documents (id, "type", status, created_at, modified_at, content, extensions)
SELECT gen_random_uuid(), 'general', 'active', now(), now(), jsonb_build_object('n', gs), '{}'::jsonb
FROM generate_series(1, $1) AS gs
        "#,
    )
    .bind(count)
    .execute(repo.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn init_bootstraps_and_repairs_governance_in_postgres() {
    let Some(url) = database_url() else {
        // No configured DB for CI/local runs.
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();

    let td = TempDir::new().unwrap();
    let governance_path = td.path().join("CONSTITUTION.md");
    std::fs::write(&governance_path, "v1").unwrap();
    let governance = FsGovernanceSource::new(td.path());

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let mut repo = repo;
    init_bundle(&mut repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let doc = repo
        .find_latest_document_by_type(DocumentType::GOVERNANCE)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(doc.status.as_str(), DocumentStatus::ACTIVE);
    assert_eq!(doc.content, serde_json::Value::String("v1".to_string()));
    let rev1 = repo
        .get_revision(doc.current_revision_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rev1.version, 1);

    std::fs::write(&governance_path, "v2").unwrap();
    init_bundle(&mut repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let doc = repo
        .find_latest_document_by_type(DocumentType::GOVERNANCE)
        .await
        .unwrap()
        .unwrap();
    let rev2 = repo
        .get_revision(doc.current_revision_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rev2.version, 2);
    assert_eq!(rev2.parent_revision_id, Some(rev1.id));
    let parent = repo.get_revision(rev1.id).await.unwrap().unwrap();
    assert!(parent.superseded_at.is_some());

    // Uniqueness: inserting another governance document should conflict.
    let doc_id = docracy_core::DocumentId(Uuid::new_v4());
    let rev_id = docracy_core::RevisionId(Uuid::new_v4());
    let res = repo
        .create_document_with_revision(
            docracy_core::Document {
                id: doc_id,
                doc_type: DocumentType::new(DocumentType::GOVERNANCE).unwrap(),
                status: DocumentStatus::active(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                current_revision_id: Some(rev_id),
                archived_at: None,
                deleted_at: None,
                content: serde_json::Value::String("x".to_string()),
                extensions: serde_json::Map::new(),
            },
            docracy_core::DocumentRevision {
                id: rev_id,
                document_id: doc_id,
                version: 1,
                parent_revision_id: None,
                created_at: chrono::Utc::now(),
                superseded_at: None,
                content: serde_json::Value::String("x".to_string()),
                extensions: serde_json::Map::new(),
            },
        )
        .await;
    assert!(matches!(res, Err(docracy_core::RepoError::Conflict)));

    // Sanity: core service flows work.
    let created = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"a": 1}),
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
            content: Some(json!({"a": 2})),
            extensions: None,
            status: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.new_revision.version, 2);

    // Query (content keyword) works.
    let created = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"msg": "hello world"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let out = query_documents(
        &repo,
        QueryInput {
            query: Some("hello".to_string()),
            sql: None,
            timeout_ms: None,
            where_: Map::new(),
            order_by: vec![],
            select: vec!["id".to_string()],
            limit: Some(10),
            cursor: None,
        },
    )
    .await
    .unwrap();
    let found_ids: Vec<String> = out
        .rows
        .iter()
        .map(|r| r.get("id").unwrap().as_str().unwrap().to_string())
        .collect();
    assert!(found_ids.contains(&created.document.id.to_string()));

    let archived = update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: None,
            extensions: None,
            status: Some(DocumentStatus::ARCHIVED.to_string()),
        },
    )
    .await
    .unwrap();

    let mut where_ = Map::new();
    where_.insert("archived".to_string(), json!(true));
    let archived_out = query_documents(
        &repo,
        QueryInput {
            query: None,
            sql: None,
            timeout_ms: None,
            where_,
            order_by: vec![],
            select: vec!["id".to_string(), "status".to_string()],
            limit: Some(10),
            cursor: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(archived_out.rows.len(), 1);
    assert_eq!(
        archived_out.rows[0].get("id").unwrap().as_str().unwrap(),
        created.document.id.to_string()
    );
    assert_eq!(
        archived_out.rows[0].get("status").unwrap(),
        &json!(DocumentStatus::ARCHIVED)
    );
    assert_eq!(archived.document.status.as_str(), DocumentStatus::ARCHIVED);
}

#[derive(sqlx::FromRow, Debug)]
struct VectorMirrorQueueRow {
    workspace_id: Uuid,
    document_id: Uuid,
    revision_id: Uuid,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    embedding_dimension: i32,
    embedding: serde_json::Value,
}

async fn vector_mirror_queue_rows(repo: &PgRepository, document_id: Uuid) -> Vec<VectorMirrorQueueRow> {
    sqlx::query_as::<_, VectorMirrorQueueRow>(
        r#"
SELECT workspace_id, document_id, revision_id, archived_at, deleted_at, embedding_dimension, embedding
FROM vector_mirror_queue
WHERE document_id = $1
ORDER BY workspace_id ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(repo.pool())
    .await
    .unwrap()
}

async fn vector_mirror_queue_rows_for_workspace(
    repo: &PgRepository,
    workspace_id: Uuid,
) -> Vec<VectorMirrorQueueRow> {
    sqlx::query_as::<_, VectorMirrorQueueRow>(
        r#"
SELECT workspace_id, document_id, revision_id, archived_at, deleted_at, embedding_dimension, embedding
FROM vector_mirror_queue
WHERE workspace_id = $1
ORDER BY document_id ASC
        "#,
    )
    .bind(workspace_id)
    .fetch_all(repo.pool())
    .await
    .unwrap()
}

#[tokio::test]
async fn vector_mirror_queue_tracks_current_snapshot_in_place() {
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
            extensions: serde_json::Map::from_iter([( 
                "embedding".to_string(),
                json!([0.25, 0.5, 0.75]),
            )]),
        },
    )
    .await
    .unwrap();

    let rows = vector_mirror_queue_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].workspace_id, workspace_id);
    assert_eq!(rows[0].document_id, created.document.id.0);
    assert_eq!(rows[0].revision_id, created.revision.id.0);
    assert_eq!(rows[0].embedding_dimension, 3);

    let updated = update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: Some(json!({"body": "beta"})),
            extensions: Some(serde_json::Map::from_iter([(
                "embedding".to_string(),
                json!([1.0, 2.0, 3.0, 4.0]),
            )])),
            status: Some(DocumentStatus::ARCHIVED.to_string()),
        },
    )
    .await
    .unwrap();

    let rows = vector_mirror_queue_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].revision_id, updated.new_revision.id.0);
    assert!(rows[0].archived_at.is_some());
    assert!(rows[0].deleted_at.is_none());
    assert_eq!(rows[0].embedding_dimension, 4);
    assert_eq!(rows[0].embedding, json!([1.0, 2.0, 3.0, 4.0]));

    let restored = update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: updated.new_revision.id,
            content: Some(json!({"body": "gamma"})),
            extensions: Some(serde_json::Map::from_iter([(
                "embedding".to_string(),
                json!([9.0, 8.0, 7.0]),
            )])),
            status: Some(DocumentStatus::ACTIVE.to_string()),
        },
    )
    .await
    .unwrap();

    let rows = vector_mirror_queue_rows(&repo, created.document.id.0).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].revision_id, restored.new_revision.id.0);
    assert!(rows[0].archived_at.is_none());
    assert!(rows[0].deleted_at.is_none());
    assert_eq!(rows[0].embedding_dimension, 3);
    assert_eq!(rows[0].embedding, json!([9.0, 8.0, 7.0]));
}

#[tokio::test]
async fn vector_mirror_queue_keeps_workspace_rows_isolated() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_a = Uuid::new_v4();
    let workspace_b = Uuid::new_v4();

    let (mut repo_a, _schema_guard_a) = isolated_repo_scoped(&url, Some(workspace_a)).await;
    repo_a.migrate().await.unwrap();
    repo_a.create_workspace(workspace_a).await.unwrap();

    let (mut repo_b, _schema_guard_b) = isolated_repo_scoped(&url, Some(workspace_b)).await;
    repo_b.migrate().await.unwrap();
    repo_b.create_workspace(workspace_b).await.unwrap();

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let doc_a = create_document(
        &mut repo_a,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"workspace": "a"}),
            extensions: serde_json::Map::from_iter([(
                "embedding".to_string(),
                json!([0.1, 0.2]),
            )]),
        },
    )
    .await
    .unwrap();

    let doc_b = create_document(
        &mut repo_b,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"workspace": "b"}),
            extensions: serde_json::Map::from_iter([(
                "embedding".to_string(),
                json!([0.3, 0.4]),
            )]),
        },
    )
    .await
    .unwrap();

    let rows_a = vector_mirror_queue_rows(&repo_a, doc_a.document.id.0).await;
    let rows_b = vector_mirror_queue_rows(&repo_b, doc_b.document.id.0).await;

    assert_eq!(rows_a.len(), 1);
    assert_eq!(rows_a[0].workspace_id, workspace_a);
    assert_eq!(rows_b.len(), 1);
    assert_eq!(rows_b[0].workspace_id, workspace_b);

    let queue_rows_a = vector_mirror_queue_rows_for_workspace(&repo_a, workspace_a).await;
    let queue_rows_b = vector_mirror_queue_rows_for_workspace(&repo_b, workspace_b).await;

    assert_eq!(queue_rows_a.len(), 1);
    assert_eq!(queue_rows_b.len(), 1);
    assert_ne!(queue_rows_a[0].workspace_id, queue_rows_b[0].workspace_id);
}

#[tokio::test]
async fn migration_enforces_same_document_revision_lineage_and_indexes() {
    let Some(url) = database_url() else {
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();

    assert_index_exists(&repo, "documents_created_at_id_idx").await;
    assert_index_exists(&repo, "documents_modified_at_id_idx").await;
    assert_index_exists(&repo, "documents_single_governance_uq").await;
    assert_index_absent(&repo, "documents_single_constitution_uq").await;
    assert_index_exists(&repo, "documents_workspace_created_at_id_idx").await;
    assert_index_exists(&repo, "documents_workspace_modified_at_id_idx").await;
    assert_index_exists(&repo, "documents_workspace_type_modified_at_id_idx").await;
    assert_index_exists(&repo, "document_revisions_workspace_document_id_idx").await;

    let td = TempDir::new().unwrap();
    let governance_path = td.path().join("CONSTITUTION.md");
    std::fs::write(&governance_path, "v1").unwrap();
    let governance = FsGovernanceSource::new(td.path());
    let clock = SystemClock;
    let ids = UuidV4Generator;

    let mut repo = repo;
    init_bundle(&mut repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let first = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"doc": 1}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let second = create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"doc": 2}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let bad_revision_id = docracy_core::RevisionId(Uuid::new_v4());
    let bad_insert = sqlx::query(
        "INSERT INTO document_revisions (id, document_id, version, parent_revision_id, created_at, content, extensions) VALUES ($1, $2, 2, $3, now(), $4, '{}'::jsonb)",
    )
    .bind(bad_revision_id.0)
    .bind(second.document.id.0)
    .bind(first.revision.id.0)
    .bind(json!({"doc": 2, "bad": true}))
    .execute(repo.pool())
    .await;

    assert!(
        bad_insert.is_err(),
        "cross-document parent revision insert should fail"
    );

    let current_revision_update =
        sqlx::query("UPDATE documents SET current_revision_id = $1 WHERE id = $2")
            .bind(first.revision.id.0)
            .bind(second.document.id.0)
            .execute(repo.pool())
            .await;

    assert!(
        current_revision_update.is_err(),
        "cross-document current_revision_id update should fail"
    );
}

#[tokio::test]
async fn raw_sql_select_returns_json_maps() {
    let Some(url) = database_url() else {
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();
    seed_documents(&repo, 3).await;

    let out = repo
        .query_raw_documents(RawQueryInput {
            sql: r#"SELECT id, "type", status, content FROM documents ORDER BY created_at ASC"#
                .to_string(),
            limit: Some(10),
            timeout_ms: Some(1000),
        })
        .await
        .unwrap();

    assert_eq!(out.total_count, 3);
    assert_eq!(out.rows.len(), 3);
    assert!(out.rows.iter().all(|row| row.contains_key("id")
        && row.contains_key("type")
        && row.contains_key("status")
        && row.contains_key("content")));
}

#[tokio::test]
async fn raw_sql_update_is_rejected_in_read_only_transaction() {
    let Some(url) = database_url() else {
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();
    seed_documents(&repo, 1).await;

    let err = repo
        .query_raw_documents(RawQueryInput {
            sql: r#"WITH updated AS (UPDATE documents SET status = 'archived' RETURNING *) SELECT * FROM updated"#.to_string(),
            limit: Some(1),
            timeout_ms: Some(1000),
        })
        .await
        .unwrap_err();

    let msg = format!("{err:?}");
    assert!(msg.contains("read-only") || msg.contains("cannot execute UPDATE"));
}

#[tokio::test]
async fn raw_sql_limit_is_clamped_to_server_ceiling() {
    let Some(url) = database_url() else {
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();
    seed_documents(&repo, 101).await;

    let out = repo
        .query_raw_documents(RawQueryInput {
            sql: r#"SELECT id, content FROM documents ORDER BY created_at ASC"#.to_string(),
            limit: Some(1000),
            timeout_ms: Some(1000),
        })
        .await
        .unwrap();

    assert_eq!(out.total_count, 101);
    assert_eq!(out.rows.len(), 100);
}

#[tokio::test]
async fn workspace_scoped_sessions_isolate_reads_queries_and_raw_sql() {
    let Some(url) = database_url() else {
        return;
    };

    let schema = unique_schema_name();
    let _schema_guard = SchemaGuard::create(&url, schema.clone()).await;
    let workspace_a = Uuid::new_v4();
    let workspace_b = Uuid::new_v4();

    let global_repo = repo_on_schema(&url, schema.clone(), None).await;
    global_repo.migrate().await.unwrap();

    let td = TempDir::new().unwrap();
    let governance_path = td.path().join("CONSTITUTION.md");
    std::fs::write(&governance_path, "global governance").unwrap();
    let governance = FsGovernanceSource::new(td.path());
    let clock = SystemClock;
    let ids = UuidV4Generator;

    let mut global_repo = global_repo;
    init_bundle(&mut global_repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let scoped_a = repo_on_schema(&url, schema.clone(), Some(workspace_a)).await;
    let scoped_b = repo_on_schema(&url, schema.clone(), Some(workspace_b)).await;

    let mut scoped_a = scoped_a;
    let mut scoped_b = scoped_b;

    let doc_a = create_document(
        &mut scoped_a,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"workspace": "a"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    let doc_b = create_document(
        &mut scoped_b,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"workspace": "b"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        workspace_id_for_document(&scoped_a, doc_a.document.id).await,
        workspace_a
    );
    assert_eq!(
        workspace_id_for_document(&scoped_b, doc_b.document.id).await,
        workspace_b
    );

    assert!(scoped_a
        .get_document(doc_b.document.id)
        .await
        .unwrap()
        .is_none());
    assert!(scoped_b
        .get_document(doc_a.document.id)
        .await
        .unwrap()
        .is_none());

    let mut where_a = Map::new();
    where_a.insert("type".to_string(), json!("general"));
    let out_a = query_documents(
        &scoped_a,
        QueryInput {
            query: None,
            sql: None,
            timeout_ms: None,
            where_: where_a,
            order_by: vec![],
            select: vec!["id".to_string()],
            limit: Some(10),
            cursor: None,
        },
    )
    .await
    .unwrap();
    let ids_a: Vec<String> = out_a
        .rows
        .iter()
        .map(|row| row.get("id").unwrap().as_str().unwrap().to_string())
        .collect();
    assert!(ids_a.contains(&doc_a.document.id.to_string()));
    assert!(!ids_a.contains(&doc_b.document.id.to_string()));

    let raw_a = scoped_a
        .query_raw_documents(RawQueryInput {
            sql: r#"SELECT id, workspace_id, content FROM documents WHERE "type" = 'general' ORDER BY created_at ASC"#.to_string(),
            limit: Some(10),
            timeout_ms: Some(1000),
        })
        .await
        .unwrap();
    assert!(raw_a
        .rows
        .iter()
        .any(|row| row.get("id").and_then(|v| v.as_str()) == Some(&doc_a.document.id.to_string())));
    assert!(!raw_a
        .rows
        .iter()
        .any(|row| row.get("id").and_then(|v| v.as_str()) == Some(&doc_b.document.id.to_string())));

    let global_gov = global_repo
        .find_latest_document_by_type(DocumentType::GOVERNANCE)
        .await
        .unwrap()
        .unwrap();
    let scoped_gov = scoped_a
        .find_latest_document_by_type(DocumentType::GOVERNANCE)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(global_gov.id, scoped_gov.id);
}

#[tokio::test]
async fn create_workspace_inserts_workspace_row() {
    let Some(url) = database_url() else {
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();

    let workspace_id = Uuid::new_v4();
    repo.create_workspace(workspace_id).await.unwrap();

    assert!(workspace_exists(&repo, workspace_id).await);
}

#[tokio::test]
async fn scoped_workspace_missing_row_surfaces_actionable_error() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_id = Uuid::new_v4();
    let (mut repo, _schema_guard) = isolated_repo_scoped(&url, Some(workspace_id)).await;
    repo.migrate().await.unwrap();

    let err = create_document(
        &mut repo,
        &SystemClock,
        &UuidV4Generator,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"hello": "world"}),
            extensions: serde_json::Map::new(),
        },
    )
    .await
    .unwrap_err();

    match err {
        docracy_core::CoreError::Repo(docracy_core::RepoError::WorkspaceNotProvisioned {
            workspace_id: err_workspace_id,
        }) => {
            assert_eq!(err_workspace_id, workspace_id);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
