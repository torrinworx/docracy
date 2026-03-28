use crate::document::{Document, DocumentStatus, DocumentType};
use crate::errors::RepoError;
use crate::ids::{DocumentId, RevisionId};
use crate::repository::Repository;
use crate::revision::DocumentRevision;
use async_trait::async_trait;
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

    fn insert_document(&mut self, doc: Document) -> Result<(), RepoError> {
        if self.documents.contains_key(&doc.id) {
            return Err(RepoError::Conflict);
        }
        if doc.doc_type.as_str() == DocumentType::CONSTITUTION
            && self
                .documents
                .values()
                .any(|d| d.doc_type.as_str() == DocumentType::CONSTITUTION)
        {
            return Err(RepoError::Conflict);
        }
        self.documents.insert(doc.id, doc);
        Ok(())
    }

    fn update_document_inner(&mut self, doc: Document) -> Result<(), RepoError> {
        if !self.documents.contains_key(&doc.id) {
            return Err(RepoError::Storage("update of missing document".to_string()));
        }
        if doc.doc_type.as_str() == DocumentType::CONSTITUTION
            && self
                .documents
                .values()
                .any(|d| d.doc_type.as_str() == DocumentType::CONSTITUTION && d.id != doc.id)
        {
            return Err(RepoError::Conflict);
        }
        self.documents.insert(doc.id, doc);
        Ok(())
    }

    fn insert_revision_inner(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        if self.revisions.contains_key(&rev.id) {
            return Err(RepoError::Conflict);
        }
        self.revisions.insert(rev.id, rev);
        Ok(())
    }

    fn update_revision_inner(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        if !self.revisions.contains_key(&rev.id) {
            return Err(RepoError::Storage("update of missing revision".to_string()));
        }
        self.revisions.insert(rev.id, rev);
        Ok(())
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn create_document_with_revision(
        &mut self,
        doc: Document,
        rev: DocumentRevision,
    ) -> Result<(), RepoError> {
        self.insert_document(doc)?;
        self.insert_revision_inner(rev)?;
        Ok(())
    }

    async fn update_document_with_revisions(
        &mut self,
        doc: Document,
        superseded: DocumentRevision,
        new_rev: DocumentRevision,
    ) -> Result<(), RepoError> {
        self.update_revision_inner(superseded)?;
        self.insert_revision_inner(new_rev)?;
        self.update_document_inner(doc)?;
        Ok(())
    }

    async fn update_document(&mut self, doc: Document) -> Result<(), RepoError> {
        self.update_document_inner(doc)
    }

    async fn get_document(&self, id: DocumentId) -> Result<Option<Document>, RepoError> {
        Ok(self.documents.get(&id).cloned())
    }

    async fn get_documents(&self, ids: &[DocumentId]) -> Result<Vec<Document>, RepoError> {
        Ok(ids
            .iter()
            .filter_map(|id| self.documents.get(id).cloned())
            .collect())
    }

    async fn find_latest_document_by_type(
        &self,
        doc_type: &str,
    ) -> Result<Option<Document>, RepoError> {
        Ok(self
            .documents
            .values()
            .filter(|d| d.doc_type.as_str() == doc_type)
            .max_by(|a, b| a.modified_at.cmp(&b.modified_at))
            .cloned())
    }

    async fn insert_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        self.insert_revision_inner(rev)
    }

    async fn update_revision(&mut self, rev: DocumentRevision) -> Result<(), RepoError> {
        self.update_revision_inner(rev)
    }

    async fn get_revision(&self, id: RevisionId) -> Result<Option<DocumentRevision>, RepoError> {
        Ok(self.revisions.get(&id).cloned())
    }

    async fn list_active_context_documents(&self) -> Result<Vec<Document>, RepoError> {
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
