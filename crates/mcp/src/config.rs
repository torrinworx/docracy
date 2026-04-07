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
    /// Whether to run SQL migrations during startup.
    pub run_migrations: bool,
    /// Optional workspace binding for the process lifetime.
    pub workspace_id: Option<Uuid>,
    /// Which transport the outer entrypoint should serve.
    pub transport: McpTransport,
}

impl McpStartupConfig {
    pub fn new(
        database_url: impl Into<String>,
        run_migrations: bool,
        workspace_id: Option<Uuid>,
        transport: McpTransport,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            run_migrations,
            workspace_id,
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

#[cfg(test)]
mod tests {
    use super::{parse_workspace_id, McpStartupConfig, McpTransport};
    use uuid::Uuid;

    #[test]
    fn startup_config_carries_workspace_id() {
        let workspace_id = Uuid::from_u128(0x1234);
        let config = McpStartupConfig::new(
            "postgres://example",
            true,
            Some(workspace_id),
            McpTransport::Stdio,
        );

        assert_eq!(config.workspace_id, Some(workspace_id));
    }

    #[test]
    fn parse_workspace_id_accepts_missing_value() {
        assert_eq!(parse_workspace_id(None).unwrap(), None);
    }

    #[test]
    fn parse_workspace_id_rejects_invalid_uuid() {
        assert!(parse_workspace_id(Some("not-a-uuid")).is_err());
    }
}
