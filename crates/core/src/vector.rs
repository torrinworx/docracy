use crate::document::{Document, DocumentStatus};
use crate::ids::{DocumentId, RevisionId};
use crate::query::DocumentQuery;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub fn canonical_embedding_source_text(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingJobRecord {
    pub workspace_id: Uuid,
    pub document_id: DocumentId,
    pub revision_id: RevisionId,
    pub embed_model: String,
    pub source_text: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorMirrorRecord {
    pub workspace_id: Uuid,
    pub document_id: DocumentId,
    pub revision_id: RevisionId,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub embedding: Vec<f32>,
}

impl VectorMirrorRecord {
    pub fn embedding_dimension(&self) -> usize {
        self.embedding.len()
    }

    pub fn from_document(
        document: &Document,
        revision_id: RevisionId,
        embedding: Vec<f32>,
    ) -> Self {
        Self {
            workspace_id: Uuid::nil(),
            document_id: document.id,
            revision_id,
            archived_at: document.archived_at,
            deleted_at: document.deleted_at,
            embedding,
        }
    }

    pub fn is_active(&self) -> bool {
        self.archived_at.is_none() && self.deleted_at.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorQueryInput {
    pub query: DocumentQuery,
    pub embedding: Vec<f32>,
}

impl VectorQueryInput {
    pub fn new(query: DocumentQuery, embedding: Vec<f32>) -> Self {
        Self { query, embedding }
    }

    pub fn embedding_dimension(&self) -> usize {
        self.embedding.len()
    }

    pub fn is_active_only(&self) -> bool {
        self.query.statuses.as_ref().is_some_and(|statuses| {
            statuses
                .iter()
                .any(|status| status == DocumentStatus::ACTIVE)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentType;
    use crate::query::DocumentQueryOrder;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn mirror_records_carry_snapshot_identity_and_embedding() {
        let record = VectorMirrorRecord {
            workspace_id: Uuid::nil(),
            document_id: DocumentId(Uuid::new_v4()),
            revision_id: RevisionId(Uuid::new_v4()),
            archived_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
            deleted_at: None,
            embedding: vec![0.1, 0.2, 0.3],
        };

        assert_eq!(record.workspace_id, Uuid::nil());
        assert_eq!(record.embedding_dimension(), 3);
        assert!(!record.is_active());
    }

    #[test]
    fn query_input_reuses_document_filters_with_embedding() {
        let query = DocumentQuery {
            query: Some("hello".to_string()),
            types: Some(vec![DocumentType::GENERAL.to_string()]),
            statuses: Some(vec![DocumentStatus::ACTIVE.to_string()]),
            archived: Some(false),
            deleted: Some(false),
            created_gte: None,
            created_lte: None,
            modified_gte: None,
            modified_lte: None,
            order: DocumentQueryOrder::ModifiedDesc,
            limit: 10,
            cursor: None,
        };

        let input = VectorQueryInput::new(query.clone(), vec![1.0, 2.0]);

        assert_eq!(input.query, query);
        assert_eq!(input.embedding_dimension(), 2);
        assert!(input.is_active_only());
    }

    #[test]
    fn embedding_job_record_carries_workspace_document_revision_model_and_tombstones() {
        let record = EmbeddingJobRecord {
            workspace_id: Uuid::new_v4(),
            document_id: DocumentId(Uuid::new_v4()),
            revision_id: RevisionId(Uuid::new_v4()),
            embed_model: "embeddinggemma".to_string(),
            source_text: canonical_embedding_source_text(&json!({"body": "alpha"})),
            archived_at: Some(Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()),
            deleted_at: None,
        };

        assert_eq!(record.embed_model, "embeddinggemma");
        assert!(record.source_text.contains("alpha"));
        assert!(record.archived_at.is_some());
        assert!(record.deleted_at.is_none());
    }

    #[test]
    fn canonical_embedding_source_text_is_stable_for_the_same_value() {
        let value = json!({"a": 1, "b": [true, false]});

        let first = canonical_embedding_source_text(&value);
        let second = canonical_embedding_source_text(&value);

        assert_eq!(first, second);
        assert_eq!(first, "{\"a\":1,\"b\":[true,false]}");
    }
}
