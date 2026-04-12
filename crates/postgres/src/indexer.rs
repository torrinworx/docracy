use super::{
    PgRepository, map_sqlx_error, qdrant_collection_name, require_ollama_embed_model,
    verify_or_pull_ollama_embed_model,
};
use docracy_core::errors::RepoError;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sqlx::types::Uuid;
use sqlx::types::chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::sync::OnceCell;
use uuid::Uuid as WorkspaceUuid;

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;
const DEFAULT_BATCH_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerConfig {
    pub ollama_url: String,
    pub ollama_embed_model: String,
    pub qdrant_url: String,
    pub poll_interval_ms: u64,
    pub batch_size: usize,
}

impl IndexerConfig {
    pub fn from_env() -> Result<Self, RepoError> {
        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
        let ollama_embed_model = require_ollama_embed_model(std::env::var("OLLAMA_EMBED_MODEL").ok())?;
        let qdrant_url =
            std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string());

        let poll_interval_ms = match std::env::var("POLL_INTERVAL_MS") {
            Ok(value) => value
                .parse::<u64>()
                .map_err(|e| RepoError::Storage(format!("invalid POLL_INTERVAL_MS: {e}")))?,
            Err(std::env::VarError::NotPresent) => DEFAULT_POLL_INTERVAL_MS,
            Err(err) => {
                return Err(RepoError::Storage(format!(
                    "invalid POLL_INTERVAL_MS: {err}"
                )));
            }
        };

        let batch_size = match std::env::var("BATCH_SIZE") {
            Ok(value) => value
                .parse::<usize>()
                .map_err(|e| RepoError::Storage(format!("invalid BATCH_SIZE: {e}")))?,
            Err(std::env::VarError::NotPresent) => DEFAULT_BATCH_SIZE,
            Err(err) => return Err(RepoError::Storage(format!("invalid BATCH_SIZE: {err}"))),
        };

        if batch_size == 0 {
            return Err(RepoError::Storage(
                "BATCH_SIZE must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            ollama_url,
            ollama_embed_model,
            qdrant_url,
            poll_interval_ms,
            batch_size,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClaimedEmbeddingJob {
    pub workspace_id: WorkspaceUuid,
    pub document_id: Uuid,
    pub revision_id: Uuid,
    pub embed_model: String,
    pub source_text: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub last_error: Option<String>,
    pub available_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub struct IndexerRuntime {
    repo: PgRepository,
    config: IndexerConfig,
    client: reqwest::Client,
    startup_ready: OnceCell<()>,
}

impl IndexerRuntime {
    pub fn startup_banner() -> &'static str {
        "docracy-indexer: workspace-agnostic embedding worker"
    }

    pub fn claim_pending_jobs_sql() -> &'static str {
        r#"
WITH claimed AS (
  SELECT
    workspace_id, document_id, revision_id, embed_model, source_text,
    archived_at, deleted_at, attempt_count, last_error, available_at,
    claimed_at, created_at, modified_at
  FROM embedding_job_queue
  WHERE available_at <= now()
    AND claimed_at IS NULL
  ORDER BY available_at ASC, modified_at DESC, document_id ASC
  FOR UPDATE SKIP LOCKED
  LIMIT $1
)
UPDATE embedding_job_queue AS q
SET claimed_at = now(),
    modified_at = now()
FROM claimed
WHERE q.workspace_id = claimed.workspace_id
  AND q.document_id = claimed.document_id
  AND q.embed_model = claimed.embed_model
RETURNING
  q.workspace_id, q.document_id, q.revision_id, q.embed_model, q.source_text,
  q.archived_at, q.deleted_at, q.attempt_count, q.last_error, q.available_at,
  q.claimed_at, q.created_at, q.modified_at
        "#
    }

    pub async fn connect_from_env(database_url: &str) -> Result<Self, RepoError> {
        let config = IndexerConfig::from_env()?;
        let repo = PgRepository::connect(database_url, config.ollama_embed_model.clone())
            .await
            .map_err(|e| RepoError::Storage(e.to_string()))?;
        Ok(Self {
            repo,
            config,
            client: reqwest::Client::new(),
            startup_ready: OnceCell::new(),
        })
    }

    pub fn new(repo: PgRepository, config: IndexerConfig) -> Self {
        Self {
            repo,
            config,
            client: reqwest::Client::new(),
            startup_ready: OnceCell::new(),
        }
    }

    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn claim_pending_jobs(&self) -> Result<Vec<ClaimedEmbeddingJob>, RepoError> {
        let limit = i64::try_from(self.config.batch_size)
            .map_err(|_| RepoError::Storage("BATCH_SIZE is too large".to_string()))?;

        let jobs = sqlx::query_as::<_, ClaimedEmbeddingJob>(Self::claim_pending_jobs_sql())
            .bind(limit)
            .fetch_all(self.repo.pool())
            .await
            .map_err(map_sqlx_error)?;

        Ok(jobs)
    }

    async fn ensure_startup_ready(&self) -> Result<(), RepoError> {
        self.startup_ready
            .get_or_try_init(|| async {
                verify_or_pull_ollama_embed_model(
                    &self.config.ollama_url,
                    &self.config.ollama_embed_model,
                )
                .await
            })
            .await
            .map(|_| ())
    }

    pub async fn process_once(&self) -> Result<usize, RepoError> {
        self.ensure_startup_ready().await?;
        let claimed = self.claim_pending_jobs().await?;

        for job in &claimed {
            self.process_claimed_job(job).await?;
        }

        Ok(claimed.len())
    }

    pub async fn run(&self) -> Result<(), RepoError> {
        loop {
            let claimed = self.process_once().await?;
            if claimed == 0 {
                tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
            }

            tokio::task::yield_now().await;
        }
    }

    async fn process_claimed_job(&self, job: &ClaimedEmbeddingJob) -> Result<(), RepoError> {
        match self.embed_job(job).await {
            Ok(embedding) => {
                self.ensure_qdrant_collection(job.workspace_id, embedding.len())
                    .await?;
                self.upsert_qdrant_point(job, &embedding).await?;
                self.complete_job(job).await?;
            }
            Err(err) => {
                self.retry_job(job, &err.to_string()).await?;
            }
        }

        Ok(())
    }

    async fn embed_job(&self, job: &ClaimedEmbeddingJob) -> Result<Vec<f32>, RepoError> {
        let url = format!("{}/api/embed", self.config.ollama_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "model": &job.embed_model,
                "input": &job.source_text,
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

    async fn ensure_qdrant_collection(
        &self,
        workspace_id: WorkspaceUuid,
        dimension: usize,
    ) -> Result<(), RepoError> {
        let collection = qdrant_collection_name(workspace_id);
        let url = format!(
            "{}/collections/{}",
            self.config.qdrant_url.trim_end_matches('/'),
            collection
        );
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RepoError::Storage(format!("qdrant collection lookup failed: {e}")))?;

        if response.status() == StatusCode::NOT_FOUND {
            let create = self
                .client
                .put(&url)
                .json(&json!({
                    "vectors": {
                        "size": dimension,
                        "distance": "Cosine",
                    }
                }))
                .send()
                .await
                .map_err(|e| RepoError::Storage(format!("qdrant collection create failed: {e}")))?;

            if !create.status().is_success() {
                let status = create.status();
                let body = create.text().await.unwrap_or_default();
                return Err(RepoError::Storage(format!(
                    "qdrant collection create failed: {status} {body}"
                )));
            }

            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RepoError::Storage(format!(
                "qdrant collection lookup failed: {status} {body}"
            )));
        }

        let info: Value = response
            .json()
            .await
            .map_err(|e| RepoError::Storage(format!("qdrant collection parse failed: {e}")))?;
        let existing_dimension = info
            .pointer("/result/config/params/vectors/size")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                RepoError::Storage("qdrant collection response missing vector size".to_string())
            })?;

        if existing_dimension != dimension as u64 {
            return Err(RepoError::Storage(format!(
                "qdrant collection dimension mismatch: existing={existing_dimension}, expected={dimension}"
            )));
        }

        Ok(())
    }

    async fn upsert_qdrant_point(
        &self,
        job: &ClaimedEmbeddingJob,
        embedding: &[f32],
    ) -> Result<(), RepoError> {
        let collection = qdrant_collection_name(job.workspace_id);
        let url = format!(
            "{}/collections/{}/points?wait=true",
            self.config.qdrant_url.trim_end_matches('/'),
            collection
        );
        let response = self
            .client
            .put(&url)
            .json(&json!({
                "points": [{
                    "id": job.document_id.to_string(),
                    "vector": embedding,
                    "payload": {
                        "workspace_id": job.workspace_id.to_string(),
                        "document_id": job.document_id.to_string(),
                        "revision_id": job.revision_id.to_string(),
                        "embed_model": &job.embed_model,
                        "archived_at": job.archived_at.map(|value| value.to_rfc3339()),
                        "deleted_at": job.deleted_at.map(|value| value.to_rfc3339()),
                        "embedding_dimension": embedding.len(),
                    }
                }]
            }))
            .send()
            .await
            .map_err(|e| RepoError::Storage(format!("qdrant upsert request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RepoError::Storage(format!(
                "qdrant upsert failed: {status} {body}"
            )));
        }

        Ok(())
    }

    async fn complete_job(&self, job: &ClaimedEmbeddingJob) -> Result<(), RepoError> {
        sqlx::query(
            r#"
DELETE FROM embedding_job_queue
WHERE workspace_id = $1 AND document_id = $2 AND embed_model = $3
            "#,
        )
        .bind(job.workspace_id)
        .bind(job.document_id)
        .bind(&job.embed_model)
        .execute(self.repo.pool())
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn retry_job(&self, job: &ClaimedEmbeddingJob, error: &str) -> Result<(), RepoError> {
        sqlx::query(
            r#"
UPDATE embedding_job_queue
SET
  attempt_count = attempt_count + 1,
  last_error = $4,
  claimed_at = NULL,
  available_at = now() + interval '5 minutes',
  modified_at = now()
WHERE workspace_id = $1 AND document_id = $2 AND embed_model = $3
            "#,
        )
        .bind(job.workspace_id)
        .bind(job.document_id)
        .bind(&job.embed_model)
        .bind(error)
        .execute(self.repo.pool())
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
