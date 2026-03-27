use crate::document::{Document, DocumentStatus, DocumentType};
use crate::errors::RepoError;
use crate::ids::{DocumentId, RevisionId};
use crate::repository::Repository;
use crate::revision::DocumentRevision;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct MemoryRepository {
    documents: HashMap<DocumentId, Document>,
    revisions: HashMap<RevisionId, DocumentRevision>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Repository for MemoryRepository {
    fn insert_document(&mut self, doc: Document) -> Result<(), RepoError> {
        if self.documents.contains_key(&doc.id) {
            return Err(RepoError::Conflict);
        }
        self.documents.insert(doc.id, doc);
        Ok(())
    }

    fn update_document(&mut self, doc: Document) -> Result<(), RepoError> {
        if !self.documents.contains_key(&doc.id) {
            return Err(RepoError::Storage("update of missing document".to_string()));
        }
        self.documents.insert(doc.id, doc);
        Ok(())
    }

    fn get_document(&self, id: DocumentId) -> Result<Option<Document>, RepoError> {
        Ok(self.documents.get(&id).cloned())
    }

    fn get_documents(&self, ids: &[DocumentId]) -> Result<Vec<Document>, RepoError> {
        Ok(ids
            .iter()
            .filter_map(|id| self.documents.get(id).cloned())
            .collect())
    }

    fn insert_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        if self.revisions.contains_key(&rev.id) {
            return Err(RepoError::Conflict);
        }
        self.revisions.insert(rev.id, rev);
        Ok(())
    }

    fn update_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        if !self.revisions.contains_key(&rev.id) {
            return Err(RepoError::Storage("update of missing revision".to_string()));
        }
        self.revisions.insert(rev.id, rev);
        Ok(())
    }

    fn get_revision(&self, id: RevisionId) -> Result<Option<DocumentRevision>, RepoError> {
        Ok(self.revisions.get(&id).cloned())
    }

    fn list_active_context_documents(&self) -> Result<Vec<Document>, RepoError> {
        Ok(self
            .documents
            .values()
            .filter(|d| {
                d.doc_type.as_str() == DocumentType::CONTEXT
                    && d.status.as_str() == DocumentStatus::ACTIVE
            })
            .cloned()
            .collect())
    }
}
