//! MCP-owned startup configuration.

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
    /// Which transport the outer entrypoint should serve.
    pub transport: McpTransport,
}

impl McpStartupConfig {
    pub fn new(
        database_url: impl Into<String>,
        run_migrations: bool,
        transport: McpTransport,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            run_migrations,
            transport,
        }
    }
}
