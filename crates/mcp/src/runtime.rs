//! Transport-agnostic runtime/bootstrap helpers.

use crate::config::McpStartupConfig;
use anyhow::Context;
use docracy_core::governance::FsGovernanceSource;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_postgres::PgRepository;

/// Fully-initialized runtime dependencies that MCP transports can share.
pub struct McpRuntime {
    pub repo: PgRepository,
    pub governance: FsGovernanceSource,
    pub clock: SystemClock,
    pub ids: UuidV4Generator,
    pub workspace_id: Option<uuid::Uuid>,
    pub task_scope: Option<String>,
}

/// Run startup migrations when configured.
pub async fn run_migrations(repo: &PgRepository, config: &McpStartupConfig) -> anyhow::Result<()> {
    if !config.run_migrations {
        return Ok(());
    }
    repo.migrate().await.context("failed to run migrations")?;
    Ok(())
}

/// Initialize Postgres repository, governance source, and deterministic helpers.
///
/// Transport entrypoints (stdio, http) should call this to avoid duplicating
/// startup logic.
pub async fn bootstrap(config: &McpStartupConfig) -> anyhow::Result<McpRuntime> {
    let repo = PgRepository::connect_scoped(&config.database_url, config.workspace_id)
        .await
        .context("failed to connect to postgres")?;
    run_migrations(&repo, config).await?;

    Ok(McpRuntime {
        repo,
        governance: FsGovernanceSource::repo_owned(),
        clock: SystemClock,
        ids: UuidV4Generator,
        workspace_id: config.workspace_id,
        task_scope: config.task_scope.clone(),
    })
}
