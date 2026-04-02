#![forbid(unsafe_code)]

//! Docracy MCP interface crate.
//!
//! This crate is intentionally thin: it owns interface/runtime concerns (configuration,
//! dependency bootstrap, transport wiring) and delegates all document/governance business
//! rules to `docracy-core`.

pub mod config;
pub mod error;
pub mod operations;
pub mod runtime;

pub use config::{McpStartupConfig, McpTransport};
pub use error::{McpError, McpErrorKind};
pub use runtime::{bootstrap, run_migrations, McpRuntime};
