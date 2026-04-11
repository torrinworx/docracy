#![forbid(unsafe_code)]

pub mod document;
pub mod errors;
pub mod governance;
pub mod ids;
pub mod memory;
pub mod query;
pub mod repository;
pub mod revision;
pub mod vector;
pub mod service;
pub mod validation;

pub use document::{Document, DocumentStatus, DocumentType, NewDocument};
pub use errors::{CoreError, GovernanceError, RepoError};
pub use governance::{FsGovernanceSource, GovernanceBundle, GovernanceFile, GovernanceSource};
pub use ids::{DocumentId, RevisionId};
pub use memory::MemoryRepository;
pub use query::{
    DocumentQuery, DocumentQueryCursor, DocumentQueryOrder, DocumentQueryResult, GuidedQueryInput,
    QueryExecution, QueryInput, QueryResult, QueryVectorInput, RawQueryInput, RawQueryResult,
};
pub use repository::Repository;
pub use revision::{DocumentRevision, NewRevision};
pub use vector::{canonical_embedding_source_text, EmbeddingJobRecord, VectorMirrorRecord, VectorQueryInput};
pub use service::{
    create_document, init_bundle, init_bundle_scoped, query_documents, read_documents,
    query_vector_documents, update_document, CreateDocumentResult, InitBundleResult,
    ReadDocumentsResult,
    UpdateDocumentInput, UpdateDocumentResult,
};
pub use validation::{ValidationError, ValidationResult};
