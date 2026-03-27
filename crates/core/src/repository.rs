use crate::document::Document;
use crate::errors::RepoError;
use crate::ids::{DocumentId, RevisionId};
use crate::revision::DocumentRevision;

/// Storage boundary for core logic.
///
/// Concrete adapters (postgres, sqlite, in-memory, etc) implement this.
pub trait Repository {
    fn insert_document(&mut self, doc: Document) -> Result<(), RepoError>;
    fn update_document(&mut self, doc: Document) -> Result<(), RepoError>;
    fn get_document(&self, id: DocumentId) -> Result<Option<Document>, RepoError>;
    fn get_documents(&self, ids: &[DocumentId]) -> Result<Vec<Document>, RepoError>;

    fn insert_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError>;
    fn update_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError>;
    fn get_revision(&self, id: RevisionId) -> Result<Option<DocumentRevision>, RepoError>;

    /// Used by Init: active type=context docs, not archived/deleted.
    fn list_active_context_documents(&self) -> Result<Vec<Document>, RepoError>;
}
