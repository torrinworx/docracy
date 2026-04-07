use crate::document::Document;
use crate::errors::RepoError;
use crate::ids::{DocumentId, RevisionId};
use crate::query::{DocumentQuery, DocumentQueryResult, RawQueryInput, RawQueryResult};
use crate::revision::DocumentRevision;
use async_trait::async_trait;

/// Storage boundary for core logic.
///
/// Concrete adapters (postgres, sqlite, in-memory, etc) implement this.
#[async_trait(?Send)]
pub trait Repository {
    async fn create_document_with_revision(
        &mut self,
        doc: Document,
        rev: DocumentRevision,
    ) -> Result<(), RepoError>;

    async fn update_document_with_revisions(
        &mut self,
        doc: Document,
        expected_head: RevisionId,
        superseded: DocumentRevision,
        new_rev: DocumentRevision,
    ) -> Result<(), RepoError>;

    async fn update_document(&mut self, doc: Document) -> Result<(), RepoError>;

    async fn get_document(&self, id: DocumentId) -> Result<Option<Document>, RepoError>;
    async fn get_documents(&self, ids: &[DocumentId]) -> Result<Vec<Document>, RepoError>;

    /// Returns the most recently modified document matching the type, regardless of status.
    async fn find_latest_document_by_type(
        &self,
        doc_type: &str,
    ) -> Result<Option<Document>, RepoError>;

    async fn insert_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError>;
    async fn update_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError>;
    async fn get_revision(&self, id: RevisionId) -> Result<Option<DocumentRevision>, RepoError>;

    /// Used by Init: active type=context docs, not archived/deleted.
    async fn list_active_context_documents(&self) -> Result<Vec<Document>, RepoError>;

    async fn query_documents(&self, query: DocumentQuery)
        -> Result<DocumentQueryResult, RepoError>;

    async fn query_raw_documents(
        &self,
        _query: RawQueryInput,
    ) -> Result<RawQueryResult, RepoError> {
        Err(RepoError::Storage(
            "raw SQL query execution is not supported by this repository".to_string(),
        ))
    }
}
