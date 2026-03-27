#![forbid(unsafe_code)]

pub mod document;
pub mod ids;
pub mod revision;
pub mod validation;

pub use document::{Document, DocumentStatus, DocumentType, NewDocument};
pub use ids::{DocumentId, RevisionId};
pub use revision::{DocumentRevision, NewRevision};
pub use validation::{ValidationError, ValidationResult};
