use docracy_core::document::{DocumentStatus, DocumentType, NewDocument};
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
    let schema = unique_schema_name();
    let _schema_guard = SchemaGuard::create(url, schema.clone()).await;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _meta| {
            let schema = schema.clone();
            Box::pin(async move {
                let sql = format!("SET search_path TO \"{}\", public", schema);
                sqlx::query(&sql).execute(conn).await?;
                Ok(())
            })
        })
        .connect(url)
        .await
        .unwrap();

    (PgRepository::new(pool), _schema_guard)
}

#[tokio::test]
async fn init_bootstraps_and_repairs_constitution_in_postgres() {
    let Some(url) = database_url() else {
        // No configured DB for CI/local runs.
        return;
    };

    let (repo, _schema_guard) = isolated_repo(&url).await;
    repo.migrate().await.unwrap();

    let td = TempDir::new().unwrap();
    let constitution_path = td.path().join("CONSTITUTION.md");
    std::fs::write(&constitution_path, "v1").unwrap();
    let governance = FsGovernanceSource::new(td.path());

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let mut repo = repo;
    init_bundle(&mut repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let doc = repo
        .find_latest_document_by_type(DocumentType::CONSTITUTION)
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

    std::fs::write(&constitution_path, "v2").unwrap();
    init_bundle(&mut repo, &governance, &clock, &ids)
        .await
        .unwrap();

    let doc = repo
        .find_latest_document_by_type(DocumentType::CONSTITUTION)
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

    // Uniqueness: inserting another constitution should conflict.
    let doc_id = docracy_core::DocumentId(Uuid::new_v4());
    let rev_id = docracy_core::RevisionId(Uuid::new_v4());
    let res = repo
        .create_document_with_revision(
            docracy_core::Document {
                id: doc_id,
                doc_type: DocumentType::new(DocumentType::CONSTITUTION).unwrap(),
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
    assert_eq!(archived_out.rows[0].get("id").unwrap().as_str().unwrap(), created.document.id.to_string());
    assert_eq!(archived_out.rows[0].get("status").unwrap(), &json!(DocumentStatus::ARCHIVED));
    assert_eq!(archived.document.status.as_str(), DocumentStatus::ARCHIVED);
}
