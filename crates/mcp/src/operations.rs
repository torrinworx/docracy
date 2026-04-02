//! Thin Docracy-to-MCP delegation layer.
//!
//! These helpers are the stable interface boundary that transport/tool code will call.
//! They intentionally delegate directly to `docracy_core` use-cases so that business
//! rules remain single-sourced in the core crate.

use crate::error::McpError;
use crate::runtime::McpRuntime;
use docracy_core::ids::DocumentId;
use docracy_core::query::{QueryInput, QueryResult};
use docracy_core::service::{
    CreateDocumentResult, InitBundleResult, ReadDocumentsResult, UpdateDocumentInput,
    UpdateDocumentResult,
};
use docracy_core::NewDocument;

/// Initialize the governance bundle and ensure required seed state exists.
pub async fn init_bundle(runtime: &mut McpRuntime) -> Result<InitBundleResult, McpError> {
    docracy_core::init_bundle(
        &mut runtime.repo,
        &runtime.governance,
        &runtime.clock,
        &runtime.ids,
    )
    .await
    .map_err(McpError::from_core)
}

/// Create a new document + initial revision.
pub async fn create_document(
    runtime: &mut McpRuntime,
    input: NewDocument,
) -> Result<CreateDocumentResult, McpError> {
    docracy_core::create_document(&mut runtime.repo, &runtime.clock, &runtime.ids, input)
        .await
        .map_err(McpError::from_core)
}

/// Read specific documents by ID.
pub async fn read_documents(
    runtime: &McpRuntime,
    ids: &[DocumentId],
) -> Result<ReadDocumentsResult, McpError> {
    docracy_core::read_documents(&runtime.repo, ids)
        .await
        .map_err(McpError::from_core)
}

/// Query documents (search + filtering + projection) using the shipped core contract.
pub async fn query_documents(
    runtime: &McpRuntime,
    input: QueryInput,
) -> Result<QueryResult, McpError> {
    docracy_core::query_documents(&runtime.repo, input)
        .await
        .map_err(McpError::from_core)
}

/// Update a document by creating a new revision, guarded by expected-head OCC.
pub async fn update_document(
    runtime: &mut McpRuntime,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResult, McpError> {
    docracy_core::update_document(&mut runtime.repo, &runtime.clock, &runtime.ids, input)
        .await
        .map_err(McpError::from_core)
}
