//! rmcp tool surface for Docracy operations.
//!
//! This module exposes the shipped Init/Create/Read/Query/Update contract as MCP tools.
//! Tool handlers intentionally delegate through `crate::operations` so business rules
//! remain single-sourced in `docracy-core`.

use crate::error::{McpError, McpErrorKind};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

/// rmcp server handler that routes MCP tool calls to Docracy operations.
#[derive(Clone)]
pub struct DocracyMcpServer {
    pub tool_router: ToolRouter<Self>,
    pub runtime: Arc<tokio::sync::Mutex<Option<crate::runtime::McpRuntime>>>,
}

impl DocracyMcpServer {
    pub fn new_unbootstrapped() -> Self {
        Self {
            tool_router: Self::tool_router(),
            runtime: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub fn new(runtime: crate::runtime::McpRuntime) -> Self {
        Self {
            tool_router: Self::tool_router(),
            runtime: Arc::new(tokio::sync::Mutex::new(Some(runtime))),
        }
    }

    async fn runtime_mut(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<crate::runtime::McpRuntime>>, rmcp::model::ErrorData>
    {
        Ok(self.runtime.lock().await)
    }

    fn runtime_missing() -> rmcp::model::ErrorData {
        McpError::new(
            McpErrorKind::SetupError,
            "runtime not initialized (start server with a bootstrapped McpRuntime)",
        )
        .to_error_data()
    }
}

#[rmcp::tool_router]
impl DocracyMcpServer {
    /// Initialize the governance bundle and return active context documents.
    #[rmcp::tool]
    pub async fn init(&self) -> Result<Content, rmcp::model::ErrorData> {
        let mut guard = self.runtime_mut().await?;
        let runtime = guard.as_mut().ok_or_else(Self::runtime_missing)?;

        let out = crate::operations::init_bundle_runtime(runtime)
            .await
            .map_err(|e| e.to_error_data())?;

        let governance_files: Vec<Value> = out
            .governance
            .files
            .into_iter()
            .map(|f| json!({"name": f.name, "content": f.content}))
            .collect();

        let out = json!({
            "governance": {"files": governance_files},
            "context_documents": out.context_documents,
        });

        Content::json(out)
    }

    /// Create a document (JSON contract aligned with the CLI).
    #[rmcp::tool]
    pub async fn create(
        &self,
        Parameters(args): Parameters<CreateArgs>,
    ) -> Result<Content, rmcp::model::ErrorData> {
        let doc_type = docracy_core::document::DocumentType::new(args.doc_type)
            .map_err(|e| McpError::new(McpErrorKind::ValidationError, e.to_string()).to_error_data())?;

        let input = docracy_core::NewDocument {
            doc_type,
            content: object_to_value(args.content),
            extensions: args.extensions.into_iter().collect(),
        };
        input
            .validate()
            .map_err(|e| McpError::new(McpErrorKind::ValidationError, e.to_string()).to_error_data())?;

        let mut guard = self.runtime_mut().await?;
        let runtime = guard.as_mut().ok_or_else(Self::runtime_missing)?;
        let out = crate::operations::create_document_runtime(runtime, input)
            .await
            .map_err(|e| e.to_error_data())?;

        Content::json(json!({"document": out.document, "revision": out.revision}))
    }

    /// Read documents by id (JSON contract aligned with the CLI).
    #[rmcp::tool]
    pub async fn read(
        &self,
        Parameters(args): Parameters<ReadArgs>,
    ) -> Result<Content, rmcp::model::ErrorData> {
        let ids = parse_document_ids(args.ids)?;

        let guard = self.runtime_mut().await?;
        let runtime = guard.as_ref().ok_or_else(Self::runtime_missing)?;

        let out = crate::operations::read_documents_runtime(runtime, &ids)
            .await
            .map_err(|e| e.to_error_data())?;

        Content::json(json!({"documents": out.documents, "missing_ids": out.missing_ids}))
    }

    /// Query documents (JSON contract aligned with the CLI/core).
    #[rmcp::tool]
    pub async fn query(
        &self,
        Parameters(args): Parameters<QueryArgs>,
    ) -> Result<Content, rmcp::model::ErrorData> {
        let core = args.into_core();

        let guard = self.runtime_mut().await?;
        let runtime = guard.as_ref().ok_or_else(Self::runtime_missing)?;

        let out = crate::operations::query_documents_runtime(runtime, core)
            .await
            .map_err(|e| e.to_error_data())?;

        Content::json(out)
    }

    /// Update a document by creating a new revision (expected-head OCC enforced in core).
    #[rmcp::tool]
    pub async fn update(
        &self,
        Parameters(args): Parameters<UpdateArgs>,
    ) -> Result<Content, rmcp::model::ErrorData> {
        let input = args.into_core()?;

        let mut guard = self.runtime_mut().await?;
        let runtime = guard.as_mut().ok_or_else(Self::runtime_missing)?;

        let out = crate::operations::update_document_runtime(runtime, input)
            .await
            .map_err(|e| e.to_error_data())?;

        Content::json(json!({
            "document": out.document,
            "new_revision": out.new_revision,
            "superseded_revision": out.superseded_revision,
        }))
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for DocracyMcpServer {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateArgs {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub content: BTreeMap<String, Value>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadArgs {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct QueryArgs {
    pub query: Option<String>,

    #[serde(rename = "where", default)]
    pub where_: BTreeMap<String, Value>,

    #[serde(default)]
    pub order_by: Vec<QueryOrderByArgs>,

    #[serde(default)]
    pub select: Vec<String>,

    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct QueryOrderByArgs {
    pub field: String,
    pub direction: OrderByDirection,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OrderByDirection {
    Asc,
    Desc,
}

impl OrderByDirection {
    fn as_str(&self) -> &'static str {
        match self {
            OrderByDirection::Asc => "asc",
            OrderByDirection::Desc => "desc",
        }
    }
}

impl QueryArgs {
    fn into_core(self) -> docracy_core::query::QueryInput {
        docracy_core::query::QueryInput {
            query: self.query,
            where_: self.where_.into_iter().collect(),
            order_by: self
                .order_by
                .into_iter()
                .map(|ob| docracy_core::query::QueryOrderByInput {
                    field: ob.field,
                    direction: ob.direction.as_str().to_string(),
                })
                .collect(),
            select: self.select,
            limit: self.limit,
            cursor: self.cursor,
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateArgs {
    pub id: String,
    #[serde(alias = "expected_head")]
    pub expected_revision: String,
    pub content: Option<BTreeMap<String, Value>>,
    pub extensions: Option<BTreeMap<String, Value>>,
    pub status: Option<String>,
}

impl UpdateArgs {
    fn into_core(self) -> Result<docracy_core::service::UpdateDocumentInput, rmcp::model::ErrorData> {
        let id = parse_document_id(&self.id)?;
        let expected_head = parse_revision_id(&self.expected_revision)?;

        Ok(docracy_core::service::UpdateDocumentInput {
            id,
            expected_head,
            content: self.content.map(object_to_value),
            extensions: self.extensions.map(|m| m.into_iter().collect()),
            status: self.status,
        })
    }
}

fn object_to_value(map: BTreeMap<String, Value>) -> Value {
    Value::Object(map.into_iter().collect())
}

fn parse_document_id(input: &str) -> Result<docracy_core::ids::DocumentId, rmcp::model::ErrorData> {
    serde_json::from_value::<docracy_core::ids::DocumentId>(Value::String(input.to_string()))
        .map_err(|_| {
            McpError::new(McpErrorKind::ValidationError, format!("invalid uuid: {input}"))
                .to_error_data()
        })
}

fn parse_revision_id(input: &str) -> Result<docracy_core::ids::RevisionId, rmcp::model::ErrorData> {
    serde_json::from_value::<docracy_core::ids::RevisionId>(Value::String(input.to_string()))
        .map_err(|_| {
            McpError::new(McpErrorKind::ValidationError, format!("invalid uuid: {input}"))
                .to_error_data()
        })
}

fn parse_document_ids(
    ids: Vec<String>,
) -> Result<Vec<docracy_core::ids::DocumentId>, rmcp::model::ErrorData> {
    ids.into_iter().map(|raw| parse_document_id(&raw)).collect()
}
