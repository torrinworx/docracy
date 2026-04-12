//! MCP-owned startup configuration.

use uuid::Uuid;

/// Which transport the MCP server should run on.
///
/// Phase 1 only needs a selection enum so future stdio/HTTP entrypoints share a
/// single configuration surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransport {
    /// Subprocess/stdin-stdout based transport.
    Stdio,
    /// Streamable HTTP transport.
    Http,
}

/// MCP startup configuration owned by the interface crate.
///
/// This keeps runtime concerns (connection, governance source, migration policy)
/// out of per-transport request handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpStartupConfig {
    /// Postgres connection string.
    pub database_url: String,
    /// Ollama base URL for model preflight.
    pub ollama_url: String,
    /// Ollama embedding model configured for this process.
    pub ollama_embed_model: String,
    /// Whether to run SQL migrations during startup.
    pub run_migrations: bool,
    /// Optional workspace binding for the process lifetime.
    pub workspace_id: Option<Uuid>,
    /// Optional task-scoped init selector for Init convenience fields.
    pub task_scope: Option<String>,
    /// Which transport the outer entrypoint should serve.
    pub transport: McpTransport,
}

impl McpStartupConfig {
    pub fn new(
        database_url: impl Into<String>,
        ollama_url: impl Into<String>,
        ollama_embed_model: impl Into<String>,
        run_migrations: bool,
        workspace_id: Option<Uuid>,
        task_scope: Option<String>,
        transport: McpTransport,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            ollama_url: ollama_url.into(),
            ollama_embed_model: ollama_embed_model.into(),
            run_migrations,
            workspace_id,
            task_scope,
            transport,
        }
    }
}

/// Parse an optional workspace binding from an environment value.
pub fn parse_workspace_id(value: Option<&str>) -> Result<Option<Uuid>, uuid::Error> {
    match value {
        Some(value) => Uuid::parse_str(value).map(Some),
        None => Ok(None),
    }
}

/// Parse an optional task scope selector from an environment value.
pub fn parse_task_scope(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_task_scope, parse_workspace_id, McpStartupConfig, McpTransport};
    use uuid::Uuid;

    #[test]
    fn startup_config_carries_workspace_id() {
        let workspace_id = Uuid::from_u128(0x1234);
        let config = McpStartupConfig::new(
            "postgres://example",
            "http://127.0.0.1:11434",
            "embeddinggemma",
            true,
            Some(workspace_id),
            Some("task/alpha".to_string()),
            McpTransport::Stdio,
        );

        assert_eq!(config.workspace_id, Some(workspace_id));
        assert_eq!(config.task_scope.as_deref(), Some("task/alpha"));
        assert_eq!(config.ollama_url, "http://127.0.0.1:11434");
        assert_eq!(config.ollama_embed_model, "embeddinggemma");
    }

    #[test]
    fn parse_workspace_id_accepts_missing_value() {
        assert_eq!(parse_workspace_id(None).unwrap(), None);
    }

    #[test]
    fn parse_workspace_id_rejects_invalid_uuid() {
        assert!(parse_workspace_id(Some("not-a-uuid")).is_err());
    }

    #[test]
    fn parse_task_scope_trims_value() {
        assert_eq!(
            parse_task_scope(Some("  task/alpha  ")),
            Some("task/alpha".to_string())
        );
    }

    #[test]
    fn parse_task_scope_discards_empty_value() {
        assert_eq!(parse_task_scope(Some("   ")), None);
        assert_eq!(parse_task_scope(None), None);
    }
}
