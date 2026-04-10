use super::{map_sqlx_error, PgRepository};
use docracy_core::errors::RepoError;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use std::time::Duration;
use uuid::Uuid as WorkspaceUuid;

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_EMBED_MODEL: &str = "embeddinggemma";
const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;
const DEFAULT_BATCH_SIZE: usize = 16;
const GLOBAL_WORKSPACE_ID: WorkspaceUuid = WorkspaceUuid::from_u128(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerConfig {
    pub workspace_id: WorkspaceUuid,
    pub ollama_url: String,
    pub ollama_embed_model: String,
    pub qdrant_url: String,
    pub poll_interval_ms: u64,
    pub batch_size: usize,
}

impl IndexerConfig {
    pub fn from_env() -> Result<Self, RepoError> {
        let workspace_id = std::env::var("WORKSPACE_ID")
            .map_err(|_| RepoError::Storage("WORKSPACE_ID is required".to_string()))
            .and_then(|value| {
                WorkspaceUuid::parse_str(&value)
                    .map_err(|e| RepoError::Storage(format!("invalid WORKSPACE_ID: {e}")))
            })?;

        if workspace_id.is_nil() {
            return Err(RepoError::Storage(
                "WORKSPACE_ID must not be the nil UUID".to_string(),
            ));
        }

        let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
        let ollama_embed_model = std::env::var("OLLAMA_EMBED_MODEL")
            .unwrap_or_else(|_| DEFAULT_OLLAMA_EMBED_MODEL.to_string());
        let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string());

        let poll_interval_ms = match std::env::var("POLL_INTERVAL_MS") {
            Ok(value) => value
                .parse::<u64>()
                .map_err(|e| RepoError::Storage(format!("invalid POLL_INTERVAL_MS: {e}")))?,
            Err(std::env::VarError::NotPresent) => DEFAULT_POLL_INTERVAL_MS,
            Err(err) => {
                return Err(RepoError::Storage(format!("invalid POLL_INTERVAL_MS: {err}")))
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
            workspace_id,
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
}

impl IndexerRuntime {
    pub fn startup_banner() -> &'static str {
        "docracy-indexer: workspace-scoped embedding worker"
    }

    pub fn claim_pending_jobs_sql() -> &'static str {
        r#"
WITH claimed AS (
  SELECT
    workspace_id, document_id, revision_id, embed_model, source_text,
    archived_at, deleted_at, attempt_count, last_error, available_at,
    claimed_at, created_at, modified_at
  FROM embedding_job_queue
  WHERE workspace_id = coalesce(current_setting('docracy.workspace_id', true)::uuid, '00000000-0000-0000-0000-000000000000'::uuid)
    AND available_at <= now()
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
        let repo = PgRepository::connect_scoped(database_url, Some(config.workspace_id))
            .await
            .map_err(|e| RepoError::Storage(e.to_string()))?;
        Ok(Self { repo, config })
    }

    pub fn new(repo: PgRepository, config: IndexerConfig) -> Self {
        Self { repo, config }
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

    pub async fn run(&self) -> Result<(), RepoError> {
        loop {
            let claimed = self.claim_pending_jobs().await?;
            if claimed.is_empty() {
                tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                continue;
            }

            // Task 2 completes the Ollama/Qdrant pipeline. For now, keep the lease/drain loop alive.
            tokio::task::yield_now().await;
        }
    }
}

pub fn default_workspace_id() -> WorkspaceUuid {
    GLOBAL_WORKSPACE_ID
}
