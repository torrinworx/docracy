//! Thin Docracy-to-MCP delegation layer.
//!
//! These helpers are the stable interface boundary that transport/tool code will call.
//! They intentionally delegate directly to `docracy_core` use-cases so that business
//! rules remain single-sourced in the core crate.

use crate::error::McpError;
use crate::runtime::McpRuntime;
use docracy_core::ids::DocumentId;
use docracy_core::query::{QueryInput, QueryResult, QueryVectorInput};
use docracy_core::repository::Repository;
use docracy_core::service::{Clock, IdGenerator};
use docracy_core::service::{
    CreateDocumentResult, InitBundleResult, ReadDocumentsResult, UpdateDocumentInput,
    UpdateDocumentResult,
};
use docracy_core::{GovernanceSource, NewDocument};

/// Initialize the governance bundle and ensure required seed state exists.
pub async fn init_bundle(
    repo: &mut dyn Repository,
    governance: &dyn GovernanceSource,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
) -> Result<InitBundleResult, McpError> {
    docracy_core::init_bundle(repo, governance, clock, ids)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`init_bundle`].
pub async fn init_bundle_runtime(runtime: &mut McpRuntime) -> Result<InitBundleResult, McpError> {
    docracy_core::init_bundle_scoped(
        &mut runtime.repo,
        &runtime.governance,
        &runtime.clock,
        &runtime.ids,
        runtime.task_scope.as_deref(),
    )
    .await
    .map_err(McpError::from_core)
}

/// Create a new document + initial revision.
pub async fn create_document(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    input: NewDocument,
) -> Result<CreateDocumentResult, McpError> {
    docracy_core::create_document(repo, clock, ids, input)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`create_document`].
pub async fn create_document_runtime(
    runtime: &mut McpRuntime,
    input: NewDocument,
) -> Result<CreateDocumentResult, McpError> {
    create_document(&mut runtime.repo, &runtime.clock, &runtime.ids, input).await
}

/// Read specific documents by ID.
pub async fn read_documents(
    repo: &dyn Repository,
    ids: &[DocumentId],
) -> Result<ReadDocumentsResult, McpError> {
    docracy_core::read_documents(repo, ids)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`read_documents`].
pub async fn read_documents_runtime(
    runtime: &McpRuntime,
    ids: &[DocumentId],
) -> Result<ReadDocumentsResult, McpError> {
    read_documents(&runtime.repo, ids).await
}

/// Query documents (search + filtering + projection) using the shipped core contract.
pub async fn query_documents(
    repo: &dyn Repository,
    input: QueryInput,
) -> Result<QueryResult, McpError> {
    docracy_core::query_documents(repo, input)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`query_documents`].
pub async fn query_documents_runtime(
    runtime: &McpRuntime,
    input: QueryInput,
) -> Result<QueryResult, McpError> {
    query_documents(&runtime.repo, input).await
}

/// Vector query documents using the shipped core contract.
pub async fn query_vector_documents(
    repo: &dyn Repository,
    input: QueryVectorInput,
) -> Result<QueryResult, McpError> {
    docracy_core::query_vector_documents(repo, input)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`query_vector_documents`].
pub async fn query_vector_documents_runtime(
    runtime: &McpRuntime,
    input: QueryVectorInput,
) -> Result<QueryResult, McpError> {
    query_vector_documents(&runtime.repo, input).await
}

/// Update a document by creating a new revision, guarded by expected-head OCC.
pub async fn update_document(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResult, McpError> {
    docracy_core::update_document(repo, clock, ids, input)
        .await
        .map_err(McpError::from_core)
}

/// Runtime convenience wrapper for [`update_document`].
pub async fn update_document_runtime(
    runtime: &mut McpRuntime,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResult, McpError> {
    update_document(&mut runtime.repo, &runtime.clock, &runtime.ids, input).await
}
