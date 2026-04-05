use crate::document::{Document, DocumentStatus, DocumentType};
use crate::errors::RepoError;
use crate::ids::{DocumentId, RevisionId};
use crate::query::{DocumentQuery, DocumentQueryCursor, DocumentQueryOrder, DocumentQueryResult};
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
        if doc.doc_type.as_str() == DocumentType::GOVERNANCE
            && self
                .documents
                .values()
                .any(|d| d.doc_type.as_str() == DocumentType::GOVERNANCE)
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
        if doc.doc_type.as_str() == DocumentType::GOVERNANCE
            && self
                .documents
                .values()
                .any(|d| d.doc_type.as_str() == DocumentType::GOVERNANCE && d.id != doc.id)
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
        expected_head: RevisionId,
        superseded: DocumentRevision,
        new_rev: DocumentRevision,
    ) -> Result<(), RepoError> {
        let Some(existing) = self.documents.get(&doc.id) else {
            return Err(RepoError::Storage("update of missing document".to_string()));
        };
        if existing.current_revision_id != Some(expected_head) {
            return Err(RepoError::Conflict);
        }
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

    async fn query_documents(
        &self,
        query: DocumentQuery,
    ) -> Result<DocumentQueryResult, RepoError> {
        let mut docs: Vec<Document> = self.documents.values().cloned().collect();

        if let Some(types) = &query.types {
            let set: std::collections::HashSet<&str> = types.iter().map(|s| s.as_str()).collect();
            docs.retain(|d| set.contains(d.doc_type.as_str()));
        }
        if let Some(statuses) = &query.statuses {
            let set: std::collections::HashSet<&str> =
                statuses.iter().map(|s| s.as_str()).collect();
            docs.retain(|d| set.contains(d.status.as_str()));
        }
        if let Some(archived) = query.archived {
            docs.retain(|d| d.archived_at.is_some() == archived);
        }
        if let Some(deleted) = query.deleted {
            docs.retain(|d| d.deleted_at.is_some() == deleted);
        }

        if let Some(gte) = query.created_gte {
            docs.retain(|d| d.created_at >= gte);
        }
        if let Some(lte) = query.created_lte {
            docs.retain(|d| d.created_at <= lte);
        }
        if let Some(gte) = query.modified_gte {
            docs.retain(|d| d.modified_at >= gte);
        }
        if let Some(lte) = query.modified_lte {
            docs.retain(|d| d.modified_at <= lte);
        }

        if let Some(q) = &query.query {
            let needle = q.to_lowercase();
            docs.retain(|d| {
                let hay = serde_json::to_string(&d.content).unwrap_or_default();
                hay.to_lowercase().contains(&needle)
            });
        }

        let total_count = docs.len() as u64;

        // Sort + keyset pagination.
        match query.order {
            DocumentQueryOrder::ModifiedDesc => {
                docs.sort_by(|a, b| (b.modified_at, b.id.0).cmp(&(a.modified_at, a.id.0)));
                if let Some(c) = &query.cursor {
                    docs.retain(|d| {
                        d.modified_at < c.ts || (d.modified_at == c.ts && d.id.0 < c.id.0)
                    });
                }
            }
            DocumentQueryOrder::ModifiedAsc => {
                docs.sort_by(|a, b| (a.modified_at, a.id.0).cmp(&(b.modified_at, b.id.0)));
                if let Some(c) = &query.cursor {
                    docs.retain(|d| {
                        d.modified_at > c.ts || (d.modified_at == c.ts && d.id.0 > c.id.0)
                    });
                }
            }
            DocumentQueryOrder::CreatedDesc => {
                docs.sort_by(|a, b| (b.created_at, b.id.0).cmp(&(a.created_at, a.id.0)));
                if let Some(c) = &query.cursor {
                    docs.retain(|d| {
                        d.created_at < c.ts || (d.created_at == c.ts && d.id.0 < c.id.0)
                    });
                }
            }
            DocumentQueryOrder::CreatedAsc => {
                docs.sort_by(|a, b| (a.created_at, a.id.0).cmp(&(b.created_at, b.id.0)));
                if let Some(c) = &query.cursor {
                    docs.retain(|d| {
                        d.created_at > c.ts || (d.created_at == c.ts && d.id.0 > c.id.0)
                    });
                }
            }
        }

        let mut page = docs;
        let has_more = page.len() > query.limit as usize;
        page.truncate(query.limit as usize);

        let next_cursor = if has_more {
            page.last().map(|d| {
                let ts = match query.order {
                    DocumentQueryOrder::ModifiedDesc | DocumentQueryOrder::ModifiedAsc => {
                        d.modified_at
                    }
                    DocumentQueryOrder::CreatedDesc | DocumentQueryOrder::CreatedAsc => {
                        d.created_at
                    }
                };
                DocumentQueryCursor { ts, id: d.id }
            })
        } else {
            None
        };

        Ok(DocumentQueryResult {
            documents: page,
            total_count,
            next_cursor,
        })
    }
}
