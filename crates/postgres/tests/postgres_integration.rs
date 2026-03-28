use docracy_core::document::{DocumentStatus, DocumentType, NewDocument};
use docracy_core::repository::Repository;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{
    create_document, init_bundle, update_document, FsGovernanceSource, UpdateDocumentInput,
};
use docracy_postgres::PgRepository;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tempfile::TempDir;
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DOCRACY_TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
}

#[tokio::test]
async fn init_bootstraps_and_repairs_constitution_in_postgres() {
    let Some(url) = database_url() else {
        // No configured DB for CI/local runs.
        return;
    };

    let schema = format!("docracy_test_{}", Uuid::new_v4().simple());

    struct SchemaGuard {
        admin_pool: sqlx::postgres::PgPool,
        schema: String,
    }
    impl Drop for SchemaGuard {
        fn drop(&mut self) {
            let schema = self.schema.clone();
            let admin_pool = self.admin_pool.clone();
            // Best-effort cleanup.
            let _ = tokio::runtime::Handle::current().block_on(async move {
                let sql = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema);
                let _ = sqlx::query(&sql).execute(&admin_pool).await;
            });
        }
    }

    // Create schema using a one-off connection.
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let create_schema_sql = format!("CREATE SCHEMA \"{}\"", schema);
    sqlx::query(&create_schema_sql)
        .execute(&admin_pool)
        .await
        .unwrap();

    let _schema_guard = SchemaGuard {
        admin_pool: admin_pool.clone(),
        schema: schema.clone(),
    };

    // Pool that always uses our schema.
    let schema_for_connect = schema.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _meta| {
            let schema = schema_for_connect.clone();
            Box::pin(async move {
                let sql = format!("SET search_path TO \"{}\", public", schema);
                sqlx::query(&sql).execute(conn).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .unwrap();

    let repo = PgRepository::new(pool);
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
            content: Some(json!({"a": 2})),
            extensions: None,
            status: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.new_revision.version, 2);
}
