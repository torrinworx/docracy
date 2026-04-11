use super::{PgRepository, map_sqlx_error, vector_embedding_from_value};
use docracy_core::errors::RepoError;
use serde_json::{Value, json};
use sqlx::types::Uuid;
use uuid::Uuid as WorkspaceUuid;

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_EMBED_MODEL: &str = "embeddinggemma";
const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
const QDRANT_VECTOR_SIZE_ENV: &str = "QDRANT_VECTOR_SIZE";

#[derive(sqlx::FromRow)]
struct VectorMirrorQueueRow {
    workspace_id: WorkspaceUuid,
    document_id: Uuid,
    revision_id: Uuid,
    archived_at: Option<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>>,
    deleted_at: Option<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>>,
    embedding_dimension: i32,
    embedding: Value,
}

pub fn qdrant_collection_name(workspace_id: WorkspaceUuid) -> String {
    format!("docracy_workspace_{workspace_id}")
}

fn qdrant_base_url() -> String {
    std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string())
}

fn qdrant_storage_error(message: impl Into<String>) -> RepoError {
    RepoError::Storage(message.into())
}

pub(crate) fn qdrant_vector_size_from_env() -> Option<usize> {
    let raw = std::env::var(QDRANT_VECTOR_SIZE_ENV).ok()?;
    let size = raw.parse::<usize>().ok()?;

    (size > 0).then_some(size)
}

fn qdrant_collection_missing_error(collection: &str) -> RepoError {
    qdrant_storage_error(format!("Qdrant collection missing: {collection}"))
}

fn is_qdrant_collection_missing_error(err: &RepoError) -> bool {
    matches!(err, RepoError::Storage(message) if message.contains("Qdrant collection missing"))
}

fn qdrant_collection_url(base_url: &str, collection: &str) -> String {
    format!(
        "{}/collections/{}",
        base_url.trim_end_matches('/'),
        collection
    )
}

fn qdrant_points_url(base_url: &str, collection: &str) -> String {
    format!(
        "{}/collections/{}/points?wait=true",
        base_url.trim_end_matches('/'),
        collection
    )
}

struct QdrantClient {
    base_url: String,
    client: reqwest::Client,
}

impl QdrantClient {
    fn with_base_url(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn new() -> Self {
        Self::with_base_url(qdrant_base_url())
    }

    async fn ensure_collection(&self, collection: &str, dimension: usize) -> Result<(), RepoError> {
        let url = qdrant_collection_url(&self.base_url, collection);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| qdrant_storage_error(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            let create = self
                .client
                .put(&url)
                .json(&json!({
                    "vectors": {
                        "size": dimension,
                        "distance": "Cosine"
                    }
                }))
                .send()
                .await
                .map_err(|e| qdrant_storage_error(e.to_string()))?;

            if !create.status().is_success() {
                let status = create.status();
                let body = create.text().await.unwrap_or_default();
                return Err(qdrant_storage_error(format!(
                    "Qdrant collection create failed: {status} {body}"
                )));
            }

            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(qdrant_storage_error(format!(
                "Qdrant collection lookup failed: {status} {body}"
            )));
        }

        let info: Value = response
            .json()
            .await
            .map_err(|e| qdrant_storage_error(e.to_string()))?;
        let existing_dimension = info
            .pointer("/result/config/params/vectors/size")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                qdrant_storage_error("Qdrant collection response missing vector size")
            })?;

        if existing_dimension != dimension as u64 {
            return Err(qdrant_storage_error(format!(
                "Qdrant collection dimension mismatch: existing={}, expected={dimension}",
                existing_dimension,
            )));
        }

        Ok(())
    }

    async fn ensure_collection_if_missing(
        &self,
        collection: &str,
        dimension: usize,
    ) -> Result<(), RepoError> {
        self.ensure_collection(collection, dimension).await
    }

    async fn upsert_point(
        &self,
        collection: &str,
        record: &VectorMirrorQueueRow,
        embedding: &[f32],
    ) -> Result<(), RepoError> {
        let url = qdrant_points_url(&self.base_url, collection);
        let response = self
            .client
            .put(&url)
            .json(&json!({
                "points": [{
                    "id": record.document_id.to_string(),
                    "vector": embedding,
                    "payload": {
                        "workspace_id": record.workspace_id.to_string(),
                        "document_id": record.document_id.to_string(),
                        "revision_id": record.revision_id.to_string(),
                        "embed_model": std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "embeddinggemma".to_string()),
                        "archived_at": record.archived_at.map(|value| value.to_rfc3339()),
                        "deleted_at": record.deleted_at.map(|value| value.to_rfc3339()),
                        "embedding_dimension": embedding.len(),
                    }
                }]
            }))
            .send()
            .await
            .map_err(|e| qdrant_storage_error(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(qdrant_storage_error(format!(
                "Qdrant upsert failed: {status} {body}"
            )));
        }

        Ok(())
    }

    async fn search_point_ids(
        &self,
        collection: &str,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<String>, RepoError> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url.trim_end_matches('/'),
            collection
        );
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "vector": embedding,
                "limit": limit,
                "with_payload": false,
                "with_vector": false,
            }))
            .send()
            .await
            .map_err(|e| qdrant_storage_error(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(qdrant_collection_missing_error(collection));
            }

            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(qdrant_storage_error(format!(
                "Qdrant search failed: {status} {body}"
            )));
        }

        let value: Value = response
            .json()
            .await
            .map_err(|e| qdrant_storage_error(e.to_string()))?;
        let points = value
            .get("result")
            .and_then(|value| value.as_array())
            .ok_or_else(|| qdrant_storage_error("Qdrant search response missing result array"))?;

        let mut ids = Vec::with_capacity(points.len());
        for point in points {
            let id = point
                .get("id")
                .and_then(|value| value.as_str())
                .ok_or_else(|| qdrant_storage_error("Qdrant search result missing point id"))?;
            ids.push(id.to_string());
        }

        Ok(ids)
    }

    async fn search_point_ids_with_recovery(
        &self,
        collection: &str,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<String>, RepoError> {
        match self.search_point_ids(collection, embedding, limit).await {
            Ok(ids) => Ok(ids),
            Err(err) if is_qdrant_collection_missing_error(&err) => {
                self.ensure_collection_if_missing(collection, embedding.len())
                    .await?;
                self.search_point_ids(collection, embedding, limit).await
            }
            Err(err) => Err(err),
        }
    }
}

pub(crate) async fn ensure_qdrant_collection(
    collection: &str,
    dimension: usize,
) -> Result<(), RepoError> {
    QdrantClient::new()
        .ensure_collection(collection, dimension)
        .await
}

pub(crate) async fn qdrant_search_point_ids(
    collection: &str,
    embedding: &[f32],
    limit: usize,
) -> Result<Vec<String>, RepoError> {
    QdrantClient::new()
        .search_point_ids_with_recovery(collection, embedding, limit)
        .await
}

pub async fn ollama_embed_text(
    input: &str,
    embed_model: Option<&str>,
) -> Result<Vec<f32>, RepoError> {
    let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
    let model = match embed_model {
        Some(model) => model.to_string(),
        None => std::env::var("OLLAMA_EMBED_MODEL")
            .unwrap_or_else(|_| DEFAULT_OLLAMA_EMBED_MODEL.to_string()),
    };

    let url = format!("{}/api/embed", ollama_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "model": model,
            "input": input,
        }))
        .send()
        .await
        .map_err(|e| RepoError::Storage(format!("ollama embed request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(RepoError::Storage(format!(
            "ollama embed failed: {status} {body}"
        )));
    }

    let value: Value = response
        .json()
        .await
        .map_err(|e| RepoError::Storage(format!("ollama embed response parse failed: {e}")))?;

    let embedding = value
        .get("embeddings")
        .and_then(|value| value.as_array())
        .and_then(|embeddings| embeddings.first())
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            RepoError::Storage("ollama embed response missing embeddings".to_string())
        })?;

    let mut vector = Vec::with_capacity(embedding.len());
    for value in embedding {
        let number = value.as_f64().ok_or_else(|| {
            RepoError::Storage("ollama embed response must contain numeric vectors".to_string())
        })?;
        vector.push(number as f32);
    }

    if vector.is_empty() {
        return Err(RepoError::Storage(
            "ollama embed response must not be empty".to_string(),
        ));
    }

    Ok(vector)
}

impl PgRepository {
    pub async fn flush_vector_mirror_queue(&mut self) -> Result<usize, RepoError> {
        let rows = sqlx::query_as::<_, VectorMirrorQueueRow>(
            r#"
SELECT workspace_id, document_id, revision_id, archived_at, deleted_at, embedding_dimension, embedding
FROM vector_mirror_queue
ORDER BY modified_at ASC, document_id ASC
            "#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(map_sqlx_error)?;

        let qdrant = QdrantClient::new();
        let mut flushed = 0usize;

        for row in rows {
            let embedding = vector_embedding_from_value(&row.embedding)?;
            if embedding.len() != row.embedding_dimension as usize {
                return Err(qdrant_storage_error(format!(
                    "vector embedding dimension mismatch: row={}, actual={}",
                    row.embedding_dimension,
                    embedding.len()
                )));
            }

            let collection = qdrant_collection_name(row.workspace_id);
            qdrant
                .ensure_collection(&collection, embedding.len())
                .await?;
            qdrant.upsert_point(&collection, &row, &embedding).await?;

            sqlx::query(
                r#"
DELETE FROM vector_mirror_queue
WHERE workspace_id = $1 AND document_id = $2
                "#,
            )
            .bind(row.workspace_id)
            .bind(row.document_id)
            .execute(self.pool())
            .await
            .map_err(map_sqlx_error)?;

            flushed += 1;
        }

        Ok(flushed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

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
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    }

    #[tokio::test]
    async fn qdrant_requests_use_workspace_scoped_collection_and_document_ids() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let captured = Arc::new(Mutex::new(Vec::<String>::new()));
        let captured_server = Arc::clone(&captured);

        let server = thread::spawn(move || {
            for request_idx in 0..4 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let request = read_http_request(&mut stream);
                captured_server
                    .lock()
                    .expect("capture lock")
                    .push(request.clone());

                match request_idx {
                    0 => respond(&mut stream, "404 Not Found", r#"{"status":"not found"}"#),
                    1 => respond(&mut stream, "200 OK", r#"{"result":{}}"#),
                    2 | 3 => respond(&mut stream, "200 OK", r#"{"result":{}}"#),
                    _ => unreachable!(),
                }
            }
        });

        let client = QdrantClient::with_base_url(format!("http://{addr}"));
        let workspace_id = WorkspaceUuid::new_v4();
        let collection = qdrant_collection_name(workspace_id);
        let row = VectorMirrorQueueRow {
            workspace_id,
            document_id: Uuid::new_v4(),
            revision_id: Uuid::new_v4(),
            archived_at: None,
            deleted_at: None,
            embedding_dimension: 3,
            embedding: json!([0.1, 0.2, 0.3]),
        };
        let embedding = vector_embedding_from_value(&row.embedding).expect("embedding");

        client
            .ensure_collection(&collection, embedding.len())
            .await
            .expect("create collection");
        client
            .upsert_point(&collection, &row, &embedding)
            .await
            .expect("first upsert");
        client
            .upsert_point(&collection, &row, &embedding)
            .await
            .expect("second upsert");

        server.join().expect("server thread");

        let captured = captured.lock().expect("captured requests");
        assert!(captured[0].starts_with(&format!("GET /collections/{collection}")));
        assert!(captured[1].starts_with(&format!("PUT /collections/{collection}")));
        assert!(
            captured[2].starts_with(&format!("PUT /collections/{collection}/points?wait=true"))
        );
        assert!(
            captured[3].starts_with(&format!("PUT /collections/{collection}/points?wait=true"))
        );
        assert!(captured[2].contains(&format!("\"id\":\"{}\"", row.document_id)));
        assert!(captured[3].contains(&format!("\"id\":\"{}\"", row.document_id)));
    }

    #[tokio::test]
    async fn qdrant_search_recreates_missing_collections_before_retrying() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let captured = Arc::new(Mutex::new(Vec::<String>::new()));
        let captured_server = Arc::clone(&captured);

        let expected_search_result_id = Uuid::new_v4().to_string();
        let search_result_id = expected_search_result_id.clone();
        let server = thread::spawn(move || {
            for request_idx in 0..4 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let request = read_http_request(&mut stream);
                captured_server
                    .lock()
                    .expect("capture lock")
                    .push(request.clone());

                match request_idx {
                    0 => respond(&mut stream, "404 Not Found", r#"{"status":"not found"}"#),
                    1 => respond(&mut stream, "404 Not Found", r#"{"status":"not found"}"#),
                    2 => respond(&mut stream, "200 OK", r#"{"result":{}}"#),
                    3 => respond(
                        &mut stream,
                        "200 OK",
                        &format!(
                            r#"{{"result":[{{"id":"{}","score":0.99}}]}}"#,
                            search_result_id
                        ),
                    ),
                    _ => unreachable!(),
                }
            }
        });

        std::env::set_var("QDRANT_URL", format!("http://{addr}"));

        let collection = qdrant_collection_name(WorkspaceUuid::new_v4());
        let ids = qdrant_search_point_ids(&collection, &[0.1, 0.2, 0.3], 1)
            .await
            .expect("search should recover");

        std::env::remove_var("QDRANT_URL");
        server.join().expect("server thread");

        assert_eq!(ids, vec![expected_search_result_id]);

        let captured = captured.lock().expect("captured requests");
        assert!(captured[0].starts_with(&format!("POST /collections/{collection}/points/search")));
        assert!(captured[1].starts_with(&format!("GET /collections/{collection}")));
        assert!(captured[2].starts_with(&format!("PUT /collections/{collection}")));
        assert!(captured[3].starts_with(&format!("POST /collections/{collection}/points/search")));
    }

    #[tokio::test]
    async fn ollama_embed_text_sends_embed_request_and_returns_embedding() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let captured = Arc::new(Mutex::new(Vec::<String>::new()));
        let captured_server = Arc::clone(&captured);

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let request = read_http_request(&mut stream);
            captured_server.lock().expect("capture lock").push(request);
            respond(&mut stream, "200 OK", r#"{"embeddings":[[0.1,0.2,0.3]]}"#);
        });

        std::env::set_var("OLLAMA_URL", format!("http://{addr}"));

        let embedding = super::ollama_embed_text("hello", Some("test-model"))
            .await
            .unwrap();

        std::env::remove_var("OLLAMA_URL");

        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);

        server.join().expect("server thread");

        let captured = captured.lock().expect("captured requests");
        assert!(captured[0].starts_with("POST /api/embed"));
        assert!(captured[0].contains("\"model\":\"test-model\""));
        assert!(captured[0].contains("\"input\":\"hello\""));
    }
}
