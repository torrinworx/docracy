//! MCP-facing error translation.
//!
//! This module exists at the interface boundary so that:
//! - `docracy_core` stays free of protocol concerns
//! - MCP tool/transport layers get stable, machine-readable error kinds + details

use docracy_core::errors::{CoreError, GovernanceError, RepoError};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum McpErrorKind {
    ValidationError,
    Conflict,
    RevisionConflict,
    WorkspaceNotProvisioned,
    StorageError,
    GovernanceIoError,
    MissingGovernance,
    DocumentNotFound,
    RevisionNotFound,
    MissingCurrentRevision,
    NoChanges,
    SetupError,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpError {
    pub kind: McpErrorKind,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl McpError {
    pub fn new(kind: McpErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn from_core(err: CoreError) -> Self {
        match err {
            CoreError::Validation(inner) => {
                McpError::new(McpErrorKind::ValidationError, inner.to_string())
            }
            CoreError::Repo(RepoError::Conflict) => {
                McpError::new(McpErrorKind::Conflict, "conflict")
            }
            CoreError::Repo(RepoError::WorkspaceNotProvisioned { workspace_id }) => {
                McpError::new(
                    McpErrorKind::WorkspaceNotProvisioned,
                    format!(
                        "workspace not provisioned: workspace_id={workspace_id}; create the workspace row first or unset WORKSPACE_ID to use the shared/global workspace"
                    ),
                )
                .with_details(json!({
                    "workspace_id": workspace_id,
                }))
            }
            CoreError::Repo(RepoError::Storage(message)) => {
                McpError::new(McpErrorKind::StorageError, message)
            }
            CoreError::Governance(GovernanceError::Io(message)) => {
                McpError::new(McpErrorKind::GovernanceIoError, message)
            }
            CoreError::Governance(GovernanceError::MissingGovernance) => McpError::new(
                McpErrorKind::MissingGovernance,
                "missing governance instructions in governance bundle",
            ),
            CoreError::DocumentNotFound => {
                McpError::new(McpErrorKind::DocumentNotFound, err.to_string())
            }
            CoreError::RevisionNotFound => {
                McpError::new(McpErrorKind::RevisionNotFound, err.to_string())
            }
            CoreError::MissingCurrentRevision => {
                McpError::new(McpErrorKind::MissingCurrentRevision, err.to_string())
            }
            CoreError::NoChanges => McpError::new(McpErrorKind::NoChanges, err.to_string()),
            CoreError::RevisionConflict { expected, actual } => {
                McpError::new(McpErrorKind::RevisionConflict, "revision conflict").with_details(
                    json!({
                        "expected": expected,
                        "actual": actual,
                    }),
                )
            }
        }
    }

    /// Translate a setup/runtime error at the MCP boundary.
    ///
    /// Today this is intentionally conservative: until transports exist, we keep
    /// a stable `setup_error` kind and surface the chain in `message`.
    pub fn from_setup(err: anyhow::Error) -> Self {
        McpError::new(McpErrorKind::SetupError, err.to_string())
    }

    /// Convert this error into rmcp's `ErrorData` with a stable machine-readable envelope.
    ///
    /// `ErrorData.data` is always JSON:
    /// `{ "kind": <snake_case McpErrorKind>, "details": <details|null> }`
    pub fn to_error_data(&self) -> rmcp::model::ErrorData {
        let kind = serde_json::to_value(self.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "internal_error".to_string());

        let data = json!({
            "kind": kind,
            "details": self.details.clone().unwrap_or(Value::Null),
        });

        use McpErrorKind::*;
        match self.kind {
            ValidationError => {
                rmcp::model::ErrorData::invalid_params(self.message.clone(), Some(data))
            }
            DocumentNotFound | RevisionNotFound | MissingCurrentRevision => {
                rmcp::model::ErrorData::invalid_request(self.message.clone(), Some(data))
            }
            WorkspaceNotProvisioned => {
                rmcp::model::ErrorData::internal_error(self.message.clone(), Some(data))
            }
            _ => rmcp::model::ErrorData::internal_error(self.message.clone(), Some(data)),
        }
    }
}
