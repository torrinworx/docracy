use crate::ids::{DocumentId, RevisionId};
use crate::validation::{ValidationError, ValidationResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type Extensions = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRevision {
    pub id: RevisionId,
    pub document_id: DocumentId,
    pub version: u32,
    pub parent_revision_id: Option<RevisionId>,

    pub created_at: DateTime<Utc>,
    pub superseded_at: Option<DateTime<Utc>>,

    pub content: Value,
    #[serde(default)]
    pub extensions: Extensions,
}

impl DocumentRevision {
    pub fn validate(&self) -> ValidationResult<()> {
        if self.version == 0 {
            return Err(ValidationError::Empty { field: "version" });
        }
        if self.content.is_null() {
            return Err(ValidationError::ContentNull);
        }
        for k in self.extensions.keys() {
            if k.is_empty() {
                return Err(ValidationError::EmptyExtensionKey);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewRevision {
    pub document_id: DocumentId,
    pub parent_revision_id: Option<RevisionId>,
    pub content: Value,

    #[serde(default)]
    pub extensions: Extensions,
}

impl NewRevision {
    pub fn validate(&self) -> ValidationResult<()> {
        if self.content.is_null() {
            return Err(ValidationError::ContentNull);
        }
        for k in self.extensions.keys() {
            if k.is_empty() {
                return Err(ValidationError::EmptyExtensionKey);
            }
        }
        Ok(())
    }
}
