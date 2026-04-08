use crate::validation::ValidationError;
use crate::RevisionId;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("conflict")]
    Conflict,

    #[error(
        "workspace not provisioned: workspace_id={workspace_id}; create it with `docracy workspace create --workspace-id {workspace_id}` or unset WORKSPACE_ID to use the shared/global workspace"
    )]
    WorkspaceNotProvisioned { workspace_id: Uuid },

    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("io error: {0}")]
    Io(String),

    #[error("missing governance instructions in governance bundle")]
    MissingGovernance,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Repo(#[from] RepoError),

    #[error(transparent)]
    Governance(#[from] GovernanceError),

    #[error("document not found")]
    DocumentNotFound,

    #[error("revision not found")]
    RevisionNotFound,

    #[error("document has no current revision")]
    MissingCurrentRevision,

    #[error("no changes provided")]
    NoChanges,

    #[error("revision conflict: expected head {expected}, found {actual:?}")]
    RevisionConflict {
        expected: RevisionId,
        actual: Option<RevisionId>,
    },
}
