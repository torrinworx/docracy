use crate::document::{Document, DocumentStatus, NewDocument};
use crate::errors::CoreError;
use crate::governance::{GovernanceBundle, GovernanceSource};
use crate::ids::{DocumentId, RevisionId};
use crate::query::{encode_cursor, project_rows, QueryInput, QueryResult};
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

pub async fn create_document(
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

    repo.create_document_with_revision(document.clone(), revision.clone())
        .await?;

    Ok(CreateDocumentResult { document, revision })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadDocumentsResult {
    pub documents: Vec<Document>,
    pub missing_ids: Vec<DocumentId>,
}

pub async fn read_documents(
    repo: &dyn Repository,
    ids: &[DocumentId],
) -> Result<ReadDocumentsResult, CoreError> {
    let docs = repo.get_documents(ids).await?;
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

pub async fn query_documents(
    repo: &dyn Repository,
    input: QueryInput,
) -> Result<QueryResult, CoreError> {
    let (query, select, applied_where) = input.parse()?;
    let out = repo.query_documents(query).await?;
    let rows = project_rows(&out.documents, &select);
    let next_cursor = out.next_cursor.as_ref().map(encode_cursor);

    Ok(QueryResult {
        rows,
        total_count: out.total_count,
        applied_where,
        next_cursor,
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

pub async fn update_document(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResult, CoreError> {
    if input.content.is_none() && input.extensions.is_none() && input.status.is_none() {
        return Err(CoreError::NoChanges);
    }

    let mut doc = repo
        .get_document(input.id)
        .await?
        .ok_or(CoreError::DocumentNotFound)?;
    let current_rev_id = doc
        .current_revision_id
        .ok_or(CoreError::MissingCurrentRevision)?;

    let mut current_rev = repo
        .get_revision(current_rev_id)
        .await?
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

    repo.update_document_with_revisions(doc.clone(), current_rev.clone(), new_rev.clone())
        .await?;

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

pub async fn init_bundle(
    repo: &mut dyn Repository,
    governance: &dyn GovernanceSource,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
) -> Result<InitBundleResult, CoreError> {
    let bundle = governance.load_bundle()?;

    // Bootstrap constitution into the DB if missing (or repair it to match the immutable file).
    let constitution_md = bundle
        .files
        .iter()
        .find(|f| f.name == "CONSTITUTION.md")
        .ok_or(crate::errors::GovernanceError::MissingConstitution)?
        .content
        .clone();
    bootstrap_constitution(repo, clock, ids, &constitution_md).await?;

    let context_documents = repo.list_active_context_documents().await?;
    Ok(InitBundleResult {
        governance: bundle,
        context_documents,
    })
}

async fn bootstrap_constitution(
    repo: &mut dyn Repository,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
    constitution_md: &str,
) -> Result<(), CoreError> {
    let expected = Value::String(constitution_md.to_string());

    let mut existing = repo
        .find_latest_document_by_type(crate::document::DocumentType::CONSTITUTION)
        .await?;

    if existing.is_none() {
        // System-created: allow reserved doc type.
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
            content: expected.clone(),
            extensions: Extensions::new(),
        };
        revision.validate()?;

        let document = Document {
            id: doc_id,
            doc_type: crate::document::DocumentType::new(
                crate::document::DocumentType::CONSTITUTION,
            )?,
            status: DocumentStatus::active(),
            created_at: now,
            modified_at: now,
            current_revision_id: Some(rev_id),
            archived_at: None,
            deleted_at: None,
            content: expected.clone(),
            extensions: Extensions::new(),
        };
        document.validate()?;

        match repo.create_document_with_revision(document, revision).await {
            Ok(()) => return Ok(()),
            Err(crate::errors::RepoError::Conflict) => {
                // Another client raced us; fall through to repair logic.
                existing = repo
                    .find_latest_document_by_type(crate::document::DocumentType::CONSTITUTION)
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }
    }

    match existing {
        None => Err(CoreError::Repo(crate::errors::RepoError::Conflict)),
        Some(doc) => {
            let needs_content = doc.content != expected;
            let needs_active = doc.status.as_str() != DocumentStatus::ACTIVE
                || doc.archived_at.is_some()
                || doc.deleted_at.is_some();
            if !needs_content && !needs_active {
                return Ok(());
            }

            update_document(
                repo,
                clock,
                ids,
                UpdateDocumentInput {
                    id: doc.id,
                    content: if needs_content { Some(expected) } else { None },
                    extensions: None,
                    status: if needs_active {
                        Some(DocumentStatus::ACTIVE.to_string())
                    } else {
                        None
                    },
                },
            )
            .await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryRepository;
    use crate::DocumentType;
    use crate::QueryInput;
    use chrono::TimeZone;
    use serde_json::json;
    use serde_json::{Map, Value};
    use tempfile::TempDir;

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

    #[derive(Debug)]
    struct SeqIds {
        docs: Vec<DocumentId>,
        revs: Vec<RevisionId>,
        doc_idx: std::cell::Cell<usize>,
        rev_idx: std::cell::Cell<usize>,
    }
    impl IdGenerator for SeqIds {
        fn new_document_id(&self) -> DocumentId {
            let i = self.doc_idx.get();
            self.doc_idx.set(i + 1);
            self.docs[i]
        }
        fn new_revision_id(&self) -> RevisionId {
            let i = self.rev_idx.get();
            self.rev_idx.set(i + 1);
            self.revs[i]
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn create_then_update_creates_revision_chain() {
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
        .await
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
        .await
        .unwrap();

        assert_eq!(updated.new_revision.version, 2);
        assert_eq!(updated.new_revision.parent_revision_id, Some(rev1));
        assert_eq!(updated.superseded_revision.superseded_at.is_some(), true);
        assert_eq!(updated.document.current_revision_id, Some(rev2));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn init_bootstraps_missing_constitution() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let clock = FixedClock(now);

        let td = TempDir::new().unwrap();
        std::fs::write(td.path().join("CONSTITUTION.md"), "hello constitution").unwrap();

        let governance = crate::FsGovernanceSource::new(td.path());

        let doc_id = DocumentId(Uuid::from_u128(10));
        let rev1 = RevisionId(Uuid::from_u128(11));
        let ids = FixedIds {
            doc: doc_id,
            revs: vec![rev1],
            idx: std::cell::Cell::new(0),
        };

        let mut repo = MemoryRepository::new();
        let out = init_bundle(&mut repo, &governance, &clock, &ids)
            .await
            .unwrap();
        assert_eq!(out.governance.files.len(), 1);

        let doc = repo
            .find_latest_document_by_type(DocumentType::CONSTITUTION)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(doc.id, doc_id);
        assert_eq!(doc.status.as_str(), DocumentStatus::ACTIVE);
        assert_eq!(doc.content, Value::String("hello constitution".to_string()));
        let rev = repo
            .get_revision(doc.current_revision_id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(rev.version, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn init_repairs_constitution_content_mismatch() {
        let t0 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let td = TempDir::new().unwrap();
        let constitution_path = td.path().join("CONSTITUTION.md");
        std::fs::write(&constitution_path, "v1").unwrap();
        let governance = crate::FsGovernanceSource::new(td.path());

        let doc_id = DocumentId(Uuid::from_u128(20));
        let rev1 = RevisionId(Uuid::from_u128(21));
        let rev2 = RevisionId(Uuid::from_u128(22));
        let ids = FixedIds {
            doc: doc_id,
            revs: vec![rev1, rev2],
            idx: std::cell::Cell::new(0),
        };

        let mut repo = MemoryRepository::new();
        init_bundle(&mut repo, &governance, &FixedClock(t0), &ids)
            .await
            .unwrap();

        std::fs::write(&constitution_path, "v2").unwrap();
        init_bundle(
            &mut repo,
            &governance,
            &FixedClock(t0 + chrono::Duration::seconds(5)),
            &ids,
        )
        .await
        .unwrap();

        let doc = repo
            .find_latest_document_by_type(DocumentType::CONSTITUTION)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(doc.id, doc_id);
        assert_eq!(doc.content, Value::String("v2".to_string()));
        let rev = repo
            .get_revision(doc.current_revision_id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(rev.version, 2);
        assert_eq!(rev.parent_revision_id, Some(rev1));
        let parent = repo.get_revision(rev1).await.unwrap().unwrap();
        assert!(parent.superseded_at.is_some());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn query_defaults_to_active_and_supports_archived_flag() {
        let t0 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let ids = SeqIds {
            docs: vec![
                DocumentId(Uuid::from_u128(100)),
                DocumentId(Uuid::from_u128(101)),
            ],
            revs: vec![
                RevisionId(Uuid::from_u128(200)),
                RevisionId(Uuid::from_u128(201)),
                RevisionId(Uuid::from_u128(202)),
            ],
            doc_idx: std::cell::Cell::new(0),
            rev_idx: std::cell::Cell::new(0),
        };

        let mut repo = MemoryRepository::new();
        let d1 = create_document(
            &mut repo,
            &FixedClock(t0),
            &ids,
            NewDocument {
                doc_type: DocumentType::new("general").unwrap(),
                content: json!({"text": "hello"}),
                extensions: Extensions::new(),
            },
        )
        .await
        .unwrap();

        let d2 = create_document(
            &mut repo,
            &FixedClock(t0 + chrono::Duration::seconds(1)),
            &ids,
            NewDocument {
                doc_type: DocumentType::new("general").unwrap(),
                content: json!({"text": "bye"}),
                extensions: Extensions::new(),
            },
        )
        .await
        .unwrap();

        // Archive the second document.
        update_document(
            &mut repo,
            &FixedClock(t0 + chrono::Duration::seconds(2)),
            &ids,
            UpdateDocumentInput {
                id: d2.document.id,
                content: None,
                extensions: None,
                status: Some(DocumentStatus::ARCHIVED.to_string()),
            },
        )
        .await
        .unwrap();

        // Default: active only.
        let out = query_documents(
            &repo,
            QueryInput {
                query: None,
                where_: Map::new(),
                order_by: vec![],
                select: vec!["id".to_string()],
                limit: Some(10),
                cursor: None,
            },
        )
        .await
        .unwrap();
        let ids: Vec<String> = out
            .rows
            .into_iter()
            .map(|r| r.get("id").unwrap().as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, vec![d1.document.id.to_string()]);

        // Archived=true: return archived docs even without status filter.
        let mut where_ = Map::new();
        where_.insert("archived".to_string(), Value::Bool(true));
        let out = query_documents(
            &repo,
            QueryInput {
                query: None,
                where_,
                order_by: vec![],
                select: vec!["id".to_string()],
                limit: Some(10),
                cursor: None,
            },
        )
        .await
        .unwrap();
        let ids: Vec<String> = out
            .rows
            .into_iter()
            .map(|r| r.get("id").unwrap().as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, vec![d2.document.id.to_string()]);
    }
}
