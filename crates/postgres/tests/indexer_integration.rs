use docracy_core::document::{DocumentType, NewDocument};
use docracy_core::create_document;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_postgres::indexer::{IndexerConfig, IndexerRuntime};
use docracy_postgres::PgRepository;
use serde_json::{json, Map, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::{DateTime, Utc};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

fn set_env(key: &str, value: &str) {
    unsafe {
        std::env::set_var(key, value);
    }
}

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

async fn seed_embedding_job(repo: &mut PgRepository, content: Value) -> docracy_core::DocumentId {
    let created = create_document(
        repo,
        &SystemClock,
        &UuidV4Generator,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content,
            extensions: Map::new(),
        },
    )
    .await
    .unwrap();

    created.document.id
}

fn read_http_request(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("set timeout");

    let mut request = Vec::new();
    let mut buf = [0u8; 1024];
    let mut header_end = None;

    loop {
        let n = stream.read(&mut buf).expect("read request");
        if n == 0 {
            break;
        }
        request.extend_from_slice(&buf[..n]);
        if header_end.is_none() {
            if let Some(pos) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                header_end = Some(pos + 4);
            }
        }

        if let Some(end) = header_end {
            let headers = String::from_utf8_lossy(&request[..end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let lower = line.to_ascii_lowercase();
                    lower
                        .strip_prefix("content-length:")
                        .map(|value| value.trim().parse::<usize>().unwrap_or(0))
                })
                .unwrap_or(0);
            if request.len() >= end + content_length {
                break;
            }
        }
    }

    String::from_utf8(request).expect("request is utf8")
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).expect("write response");
}

fn start_ollama_mock(
    response_body: &'static str,
    request_limit: usize,
) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ollama mock");
    let addr = listener.local_addr().expect("mock addr");
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured_server = Arc::clone(&captured);

    let handle = thread::spawn(move || {
        for _ in 0..request_limit {
            let (mut stream, _) = listener.accept().expect("accept request");
            let request = read_http_request(&mut stream);
            captured_server.lock().expect("capture lock").push(request.clone());
            respond(&mut stream, "200 OK", response_body);
        }
    });

    (format!("http://{addr}"), captured, handle)
}

fn start_qdrant_mock(request_limit: usize) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind qdrant mock");
    let addr = listener.local_addr().expect("mock addr");
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured_server = Arc::clone(&captured);

    let handle = thread::spawn(move || {
        for request_idx in 0..request_limit {
            let (mut stream, _) = listener.accept().expect("accept request");
            let request = read_http_request(&mut stream);
            captured_server.lock().expect("capture lock").push(request.clone());

            match request_idx {
                0 => respond(&mut stream, "404 Not Found", r#"{"status":"not found"}"#),
                1 => respond(&mut stream, "200 OK", r#"{"result":{}}"#),
                _ => respond(&mut stream, "200 OK", r#"{"result":{}}"#),
            }
        }
    });

    (format!("http://{addr}"), captured, handle)
}

#[test]
fn indexer_config_reads_worker_env() {
    set_env("WORKSPACE_ID", "00000000-0000-0000-0000-000000000001");
    set_env("OLLAMA_URL", "http://127.0.0.1:11434");
    set_env("OLLAMA_EMBED_MODEL", "embeddinggemma");
    set_env("QDRANT_URL", "http://127.0.0.1:6333");
    set_env("POLL_INTERVAL_MS", "250");
    set_env("BATCH_SIZE", "8");

    let config = IndexerConfig::from_env().expect("config should parse");

    assert_eq!(
        config.workspace_id.to_string(),
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(config.ollama_url, "http://127.0.0.1:11434");
    assert_eq!(config.ollama_embed_model, "embeddinggemma");
    assert_eq!(config.qdrant_url, "http://127.0.0.1:6333");
    assert_eq!(config.poll_interval_ms, 250);
    assert_eq!(config.batch_size, 8);
}

#[test]
fn indexer_claim_sql_uses_skip_locked_and_workspace_binding() {
    let sql = IndexerRuntime::claim_pending_jobs_sql();

    assert!(sql.contains("FOR UPDATE SKIP LOCKED"));
    assert!(sql.contains("docracy.workspace_id"));
    assert!(sql.contains("available_at <= now()"));
}

#[test]
fn indexer_entrypoint_stays_text_only() {
    let stdout = IndexerRuntime::startup_banner();

    assert!(!stdout.trim_start().starts_with('{'));
    assert!(!stdout.trim_start().starts_with('['));
}

#[tokio::test]
async fn indexer_embeds_canonical_source_text_and_upserts_qdrant() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_id = Uuid::new_v4();
    let (mut repo, _schema_guard) = isolated_repo_scoped(&url, Some(workspace_id)).await;
    repo.migrate().await.unwrap();
    repo.create_workspace(workspace_id).await.unwrap();

    let document_id = seed_embedding_job(&mut repo, json!({"body": "alpha"})).await;

    let ollama_body = r#"{"model":"embeddinggemma","embeddings":[[0.1,0.2,0.3]]}"#;
    let (ollama_url, ollama_requests, ollama_server) = start_ollama_mock(ollama_body, 1);
    let (qdrant_url, qdrant_requests, qdrant_server) = start_qdrant_mock(3);

    set_env("WORKSPACE_ID", &workspace_id.to_string());
    set_env("OLLAMA_URL", &ollama_url);
    set_env("OLLAMA_EMBED_MODEL", "embeddinggemma");
    set_env("QDRANT_URL", &qdrant_url);

    let config = IndexerConfig::from_env().unwrap();
    let runtime = IndexerRuntime::new(repo, config);

    runtime.process_once().await.unwrap();

    let requests = ollama_requests.lock().unwrap();
    assert!(requests[0].starts_with("POST /api/embed"));
    assert!(requests[0].contains("\"model\":\"embeddinggemma\""));
    assert!(requests[0].contains("\"input\":\"{\\\"body\\\":\\\"alpha\\\"}\""));

    let requests = qdrant_requests.lock().unwrap();
    assert!(requests.iter().any(|request| {
        request.starts_with(&format!("PUT /collections/docracy_workspace_{}", workspace_id))
    }));
    assert!(requests.iter().any(|request| request.contains("\"embed_model\":\"embeddinggemma\"")));
    assert!(requests.iter().any(|request| request.contains(&document_id.to_string())));

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM embedding_job_queue WHERE workspace_id = $1 AND document_id = $2",
    )
    .bind(workspace_id)
    .bind(document_id.0)
    .fetch_one(runtime.repo().pool())
    .await
    .unwrap();

    assert_eq!(remaining, 0);

    ollama_server.join().unwrap();
    qdrant_server.join().unwrap();
}

#[tokio::test]
async fn indexer_retries_failed_jobs_with_attempt_count_and_error_metadata() {
    let Some(url) = database_url() else {
        return;
    };

    let workspace_id = Uuid::new_v4();
    let (mut repo, _schema_guard) = isolated_repo_scoped(&url, Some(workspace_id)).await;
    repo.migrate().await.unwrap();
    repo.create_workspace(workspace_id).await.unwrap();

    let document_id = seed_embedding_job(&mut repo, json!({"body": "retry me"})).await;

    let (ollama_url, ollama_requests, ollama_server) = start_ollama_mock(
        r#"{"error":"boom"}"#,
        1,
    );

    set_env("WORKSPACE_ID", &workspace_id.to_string());
    set_env("OLLAMA_URL", &ollama_url);
    set_env("OLLAMA_EMBED_MODEL", "embeddinggemma");
    set_env("QDRANT_URL", "http://127.0.0.1:1");

    let config = IndexerConfig::from_env().unwrap();
    let runtime = IndexerRuntime::new(repo, config);

    runtime.process_once().await.unwrap();

    let requests = ollama_requests.lock().unwrap();
    assert!(requests[0].starts_with("POST /api/embed"));

    #[derive(sqlx::FromRow)]
    struct QueueRow {
        attempt_count: i32,
        last_error: Option<String>,
        claimed_at: Option<DateTime<Utc>>,
        available_at: DateTime<Utc>,
    }

    let row: QueueRow = sqlx::query_as::<_, QueueRow>(
        "SELECT attempt_count, last_error, claimed_at, available_at FROM embedding_job_queue WHERE workspace_id = $1 AND document_id = $2",
    )
    .bind(workspace_id)
    .bind(document_id.0)
    .fetch_one(runtime.repo().pool())
    .await
    .unwrap();

    assert_eq!(row.attempt_count, 1);
    assert!(row.last_error.as_deref().unwrap_or("").contains("boom") || row.last_error.is_some());
    assert!(row.claimed_at.is_none());
    assert!(row.available_at > Utc::now());

    ollama_server.join().unwrap();
}
