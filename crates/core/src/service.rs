use crate::document::{Document, DocumentStatus, NewDocument};
use crate::errors::CoreError;
use crate::governance::{GovernanceBundle, GovernanceSource};
use crate::ids::{DocumentId, RevisionId};
use crate::repository::Repository;
use crate::revision::DocumentRevision;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

pub type Extensions = Map<String, Value>;

pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub trait IdGenerator {
    fn new_document_id(&self) -> DocumentId;
    fn new_revision_id(&self) -> RevisionId;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UuidV4Generator;

impl IdGenerator for UuidV4Generator {
    fn new_document_id(&self) -> DocumentId {
        DocumentId(Uuid::new_v4())
    }

    fn new_revision_id(&self) -> RevisionId {
        RevisionId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateDocumentResult {
    pub document: Document,
    pub revision: DocumentRevision,
}

pub fn create_document(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    input: NewDocument,
) -> Result<CreateDocumentResult, CoreError> {
    input.validate()?;

    let now = clock.now();
    let doc_id = ids.new_document_id();
    let rev_id = ids.new_revision_id();

    let revision = DocumentRevision {
        id: rev_id,
        document_id: doc_id,
        version: 1,
        parent_revision_id: None,
        created_at: now,
        superseded_at: None,
        content: input.content.clone(),
        extensions: input.extensions.clone(),
    };
    revision.validate()?;

    let document = Document {
        id: doc_id,
        doc_type: input.doc_type,
        status: DocumentStatus::active(),
        created_at: now,
        modified_at: now,
        current_revision_id: Some(rev_id),
        archived_at: None,
        deleted_at: None,
        content: input.content,
        extensions: input.extensions,
    };
    document.validate()?;

    repo.insert_document(document.clone())?;
    repo.insert_revision(revision.clone())?;

    Ok(CreateDocumentResult { document, revision })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadDocumentsResult {
    pub documents: Vec<Document>,
    pub missing_ids: Vec<DocumentId>,
}

pub fn read_documents(
    repo: &dyn Repository,
    ids: &[DocumentId],
) -> Result<ReadDocumentsResult, CoreError> {
    let docs = repo.get_documents(ids)?;
    let mut found = std::collections::HashSet::new();
    for d in &docs {
        found.insert(d.id);
    }
    let missing_ids = ids
        .iter()
        .copied()
        .filter(|id| !found.contains(id))
        .collect();

    Ok(ReadDocumentsResult {
        documents: docs,
        missing_ids,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateDocumentResult {
    pub document: Document,
    pub new_revision: DocumentRevision,
    pub superseded_revision: DocumentRevision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateDocumentInput {
    pub id: DocumentId,
    pub content: Option<Value>,
    pub extensions: Option<Extensions>,
    pub status: Option<String>,
}

pub fn update_document(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResult, CoreError> {
    if input.content.is_none() && input.extensions.is_none() && input.status.is_none() {
        return Err(CoreError::NoChanges);
    }

    let mut doc = repo
        .get_document(input.id)?
        .ok_or(CoreError::DocumentNotFound)?;
    let current_rev_id = doc
        .current_revision_id
        .ok_or(CoreError::MissingCurrentRevision)?;

    let mut current_rev = repo
        .get_revision(current_rev_id)?
        .ok_or(CoreError::RevisionNotFound)?;

    let now = clock.now();

    // Supersede current revision.
    current_rev.superseded_at = Some(now);
    current_rev.validate()?;

    let new_rev_id = ids.new_revision_id();
    let new_content = input.content.clone().unwrap_or_else(|| doc.content.clone());
    let new_extensions = input
        .extensions
        .clone()
        .unwrap_or_else(|| doc.extensions.clone());

    let new_rev = DocumentRevision {
        id: new_rev_id,
        document_id: doc.id,
        version: current_rev.version + 1,
        parent_revision_id: Some(current_rev.id),
        created_at: now,
        superseded_at: None,
        content: new_content.clone(),
        extensions: new_extensions.clone(),
    };
    new_rev.validate()?;

    // Apply document changes.
    doc.modified_at = now;
    doc.current_revision_id = Some(new_rev_id);
    doc.content = new_content;
    doc.extensions = new_extensions;

    if let Some(status) = input.status {
        let status = crate::document::DocumentStatus::new(status)?;
        doc.status = status;

        match doc.status.as_str() {
            DocumentStatus::ACTIVE => {
                doc.archived_at = None;
                doc.deleted_at = None;
            }
            DocumentStatus::ARCHIVED => {
                doc.archived_at = Some(now);
                doc.deleted_at = None;
            }
            DocumentStatus::DELETED => {
                doc.deleted_at = Some(now);
                doc.archived_at = None;
            }
            _ => {
                // Other statuses are allowed by design, but phase-1 timestamp rules
                // only cover archived/deleted explicitly.
            }
        }
    }
    doc.validate()?;

    // Persist.
    repo.update_revision(current_rev.clone())?;
    repo.insert_revision(new_rev.clone())?;
    repo.update_document(doc.clone())?;

    Ok(UpdateDocumentResult {
        document: doc,
        new_revision: new_rev,
        superseded_revision: current_rev,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitBundleResult {
    pub governance: GovernanceBundle,
    pub context_documents: Vec<Document>,
}

pub fn init_bundle(
    repo: &dyn Repository,
    governance: &dyn GovernanceSource,
) -> Result<InitBundleResult, CoreError> {
    let bundle = governance.load_bundle()?;
    let context_documents = repo.list_active_context_documents()?;
    Ok(InitBundleResult {
        governance: bundle,
        context_documents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryRepository;
    use crate::DocumentType;
    use chrono::TimeZone;
    use serde_json::json;

    #[derive(Debug, Clone, Copy)]
    struct FixedClock(DateTime<Utc>);
    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct FixedIds {
        doc: DocumentId,
        revs: Vec<RevisionId>,
        idx: std::cell::Cell<usize>,
    }
    impl IdGenerator for FixedIds {
        fn new_document_id(&self) -> DocumentId {
            self.doc
        }
        fn new_revision_id(&self) -> RevisionId {
            let i = self.idx.get();
            self.idx.set(i + 1);
            self.revs[i]
        }
    }

    #[test]
    fn create_then_update_creates_revision_chain() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let clock = FixedClock(now);

        let doc_id = DocumentId(Uuid::from_u128(1));
        let rev1 = RevisionId(Uuid::from_u128(2));
        let rev2 = RevisionId(Uuid::from_u128(3));
        let ids = FixedIds {
            doc: doc_id,
            revs: vec![rev1, rev2],
            idx: std::cell::Cell::new(0),
        };

        let mut repo = MemoryRepository::new();
        let created = create_document(
            &mut repo,
            &clock,
            &ids,
            NewDocument {
                doc_type: DocumentType::new("general").unwrap(),
                content: json!({"a": 1}),
                extensions: Extensions::new(),
            },
        )
        .unwrap();

        assert_eq!(created.document.id, doc_id);
        assert_eq!(created.document.current_revision_id, Some(rev1));
        assert_eq!(created.revision.version, 1);

        let updated = update_document(
            &mut repo,
            &FixedClock(now + chrono::Duration::seconds(5)),
            &ids,
            UpdateDocumentInput {
                id: doc_id,
                content: Some(json!({"a": 2})),
                extensions: None,
                status: None,
            },
        )
        .unwrap();

        assert_eq!(updated.new_revision.version, 2);
        assert_eq!(updated.new_revision.parent_revision_id, Some(rev1));
        assert_eq!(updated.superseded_revision.superseded_at.is_some(), true);
        assert_eq!(updated.document.current_revision_id, Some(rev2));
    }
}
