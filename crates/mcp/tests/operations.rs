//! Integration tests for the MCP operation/delegation boundary.
//!
//! These tests run without any stdio/HTTP transport: they use the core in-memory
//! repository to ensure the MCP interface crate is a thin adapter over the shipped
//! `docracy_core` contract.

use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{DocumentType, MemoryRepository, NewDocument, UpdateDocumentInput};
use docracy_mcp::{operations, McpErrorKind};
use serde_json::{json, Map, Value};

#[tokio::test(flavor = "current_thread")]
async fn query_documents_delegates_to_core() {
    let clock = SystemClock;
    let ids = UuidV4Generator;
    let mut repo = MemoryRepository::new();

    let created = operations::create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"hello": "world"}),
            extensions: docracy_core::service::Extensions::new(),
        },
    )
    .await
    .unwrap();

    let out = operations::query_documents(
        &repo,
        docracy_core::QueryInput {
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

    assert_eq!(out.total_count, 1);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(
        out.rows[0].get("id"),
        Some(&Value::String(created.document.id.to_string()))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn update_revision_conflict_maps_to_machine_readable_details() {
    let clock = SystemClock;
    let ids = UuidV4Generator;
    let mut repo = MemoryRepository::new();

    let created = operations::create_document(
        &mut repo,
        &clock,
        &ids,
        NewDocument {
            doc_type: DocumentType::new("general").unwrap(),
            content: json!({"v": 1}),
            extensions: docracy_core::service::Extensions::new(),
        },
    )
    .await
    .unwrap();

    let updated = operations::update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: Some(json!({"v": 2})),
            extensions: None,
            status: None,
        },
    )
    .await
    .unwrap();

    let err = operations::update_document(
        &mut repo,
        &clock,
        &ids,
        UpdateDocumentInput {
            id: created.document.id,
            expected_head: created.revision.id,
            content: Some(json!({"v": 3})),
            extensions: None,
            status: None,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(err.kind, McpErrorKind::RevisionConflict);
    let details = err.details.expect("expected structured details");
    assert_eq!(
        details.get("expected"),
        Some(&serde_json::to_value(created.revision.id).unwrap())
    );
    assert_eq!(
        details.get("actual"),
        Some(&serde_json::to_value(Some(updated.new_revision.id)).unwrap())
    );
}
