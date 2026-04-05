use crate::ids::{DocumentId, RevisionId};
use crate::validation::{
    validate_mutable_document_type, validate_slug, ValidationError, ValidationResult,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentType(String);

impl DocumentType {
    pub const GOVERNANCE: &'static str = "governance";
    pub const CONTEXT: &'static str = "context";
    pub const GENERAL: &'static str = "general";
    pub const CHATS: &'static str = "chats";

    pub fn new(value: impl Into<String>) -> ValidationResult<Self> {
        let value = value.into();
        validate_slug("type", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_governance(&self) -> bool {
        self.0 == Self::GOVERNANCE
    }
}

impl TryFrom<String> for DocumentType {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DocumentType> for String {
    fn from(value: DocumentType) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentStatus(String);

impl DocumentStatus {
    pub const ACTIVE: &'static str = "active";
    pub const ARCHIVED: &'static str = "archived";
    pub const DELETED: &'static str = "deleted";
    pub const SUPERSEDED: &'static str = "superseded";

    pub fn new(value: impl Into<String>) -> ValidationResult<Self> {
        let value = value.into();
        validate_slug("status", &value)?;
        Ok(Self(value))
    }

    pub fn active() -> Self {
        Self(Self::ACTIVE.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DocumentStatus {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DocumentStatus> for String {
    fn from(value: DocumentStatus) -> Self {
        value.0
    }
}

pub type Extensions = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,

    #[serde(rename = "type")]
    pub doc_type: DocumentType,
    pub status: DocumentStatus,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,

    pub current_revision_id: Option<RevisionId>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub content: Value,
    #[serde(default)]
    pub extensions: Extensions,
}

impl Document {
    pub fn validate(&self) -> ValidationResult<()> {
        if self.content.is_null() {
            return Err(ValidationError::ContentNull);
        }
        validate_extensions(&self.extensions)?;

        if self.modified_at < self.created_at {
            return Err(ValidationError::ModifiedBeforeCreated);
        }

        // For phase 1, keep status/timestamp rules simple and strict.
        let is_archived = self.status.as_str() == DocumentStatus::ARCHIVED;
        if is_archived != self.archived_at.is_some() {
            return Err(ValidationError::StatusTimestampMismatch);
        }

        let is_deleted = self.status.as_str() == DocumentStatus::DELETED;
        if is_deleted != self.deleted_at.is_some() {
            return Err(ValidationError::StatusTimestampMismatch);
        }

        if self.archived_at.is_some() && self.deleted_at.is_some() {
            return Err(ValidationError::StatusTimestampMismatch);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewDocument {
    #[serde(rename = "type")]
    pub doc_type: DocumentType,
    pub content: Value,

    #[serde(default)]
    pub extensions: Extensions,
}

impl NewDocument {
    /// Validation appropriate for agent-supplied input.
    pub fn validate(&self) -> ValidationResult<()> {
        validate_mutable_document_type(&self.doc_type)?;
        if self.content.is_null() {
            return Err(ValidationError::ContentNull);
        }
        validate_extensions(&self.extensions)?;
        Ok(())
    }
}

fn validate_extensions(extensions: &Extensions) -> ValidationResult<()> {
    for k in extensions.keys() {
        if k.is_empty() {
            return Err(ValidationError::EmptyExtensionKey);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn document_type_validates_slug_shape() {
        assert!(DocumentType::new("general").is_ok());
        assert!(DocumentType::new("General").is_err());
        assert!(DocumentType::new("gen-eral").is_err());
        assert!(DocumentType::new("").is_err());
    }

    #[test]
    fn new_document_rejects_governance_type() {
        let nd = NewDocument {
            doc_type: DocumentType::new(DocumentType::GOVERNANCE).unwrap(),
            content: json!("hi"),
            extensions: Extensions::new(),
        };
        assert_eq!(
            nd.validate().unwrap_err(),
            ValidationError::ReservedGovernanceType
        );
    }

    #[test]
    fn document_status_requires_matching_timestamps() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let id = DocumentId(Uuid::new_v4());

        let doc = Document {
            id,
            doc_type: DocumentType::new("general").unwrap(),
            status: DocumentStatus::new(DocumentStatus::ARCHIVED).unwrap(),
            created_at: now,
            modified_at: now,
            current_revision_id: None,
            archived_at: None,
            deleted_at: None,
            content: json!({"k":"v"}),
            extensions: Extensions::new(),
        };
        assert_eq!(
            doc.validate().unwrap_err(),
            ValidationError::StatusTimestampMismatch
        );
    }
}
