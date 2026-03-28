use crate::document::{Document, DocumentStatus};
use crate::ids::DocumentId;
use crate::validation::{validate_slug, ValidationError, ValidationResult};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QueryInput {
    pub query: Option<String>,

    #[serde(rename = "where", default)]
    pub where_: Map<String, Value>,

    #[serde(default)]
    pub order_by: Vec<QueryOrderByInput>,

    #[serde(default)]
    pub select: Vec<String>,

    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryOrderByInput {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentQueryOrder {
    ModifiedDesc,
    ModifiedAsc,
    CreatedDesc,
    CreatedAsc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentQueryCursor {
    pub ts: DateTime<Utc>,
    pub id: DocumentId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentQuery {
    pub query: Option<String>,
    pub types: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub archived: Option<bool>,
    pub deleted: Option<bool>,
    pub created_gte: Option<DateTime<Utc>>,
    pub created_lte: Option<DateTime<Utc>>,
    pub modified_gte: Option<DateTime<Utc>>,
    pub modified_lte: Option<DateTime<Utc>>,
    pub order: DocumentQueryOrder,
    pub limit: u32,
    pub cursor: Option<DocumentQueryCursor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentQueryResult {
    pub documents: Vec<Document>,
    pub total_count: u64,
    pub next_cursor: Option<DocumentQueryCursor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<Map<String, Value>>,
    pub total_count: u64,
    pub applied_where: Map<String, Value>,
    pub next_cursor: Option<String>,
}

impl QueryInput {
    pub fn parse(self) -> ValidationResult<(DocumentQuery, Vec<SelectField>, Map<String, Value>)> {
        // Parse where
        let mut types: Option<Vec<String>> = None;
        let mut statuses: Option<Vec<String>> = None;
        let mut created_gte: Option<DateTime<Utc>> = None;
        let mut created_lte: Option<DateTime<Utc>> = None;
        let mut modified_gte: Option<DateTime<Utc>> = None;
        let mut modified_lte: Option<DateTime<Utc>> = None;
        let mut archived: Option<bool> = None;
        let mut deleted: Option<bool> = None;

        for (k, v) in &self.where_ {
            if k.starts_with("extensions.") {
                return Err(ValidationError::InvalidSlug { field: "where" });
            }
            match k.as_str() {
                "type" => types = Some(parse_string_or_array("where.type", v)?),
                "status" => statuses = Some(parse_string_or_array("where.status", v)?),
                "created_gte" => created_gte = Some(parse_rfc3339("where.created_gte", v)?),
                "created_lte" => created_lte = Some(parse_rfc3339("where.created_lte", v)?),
                "modified_gte" => modified_gte = Some(parse_rfc3339("where.modified_gte", v)?),
                "modified_lte" => modified_lte = Some(parse_rfc3339("where.modified_lte", v)?),
                "archived" => archived = Some(parse_bool("where.archived", v)?),
                "deleted" => deleted = Some(parse_bool("where.deleted", v)?),
                _ => return Err(ValidationError::InvalidSlug { field: "where" }),
            }
        }

        // README expectation: archived/deleted docs should not appear by default.
        // Default to status=active, unless caller explicitly asks for archived/deleted.
        let statuses = match statuses {
            Some(s) => {
                for it in &s {
                    validate_slug("where.status", it)?;
                }
                Some(s)
            }
            None => {
                if archived == Some(true) || deleted == Some(true) {
                    None
                } else {
                    Some(vec![DocumentStatus::ACTIVE.to_string()])
                }
            }
        };
        if let Some(ts) = &types {
            for t in ts {
                validate_slug("where.type", t)?;
            }
        }

        // Parse order_by
        let order = if self.order_by.is_empty() {
            DocumentQueryOrder::ModifiedDesc
        } else if self.order_by.len() == 1 {
            let ob = &self.order_by[0];
            let dir = ob.direction.to_lowercase();
            match (ob.field.as_str(), dir.as_str()) {
                ("modified", "desc") => DocumentQueryOrder::ModifiedDesc,
                ("modified", "asc") => DocumentQueryOrder::ModifiedAsc,
                ("created", "desc") => DocumentQueryOrder::CreatedDesc,
                ("created", "asc") => DocumentQueryOrder::CreatedAsc,
                _ => return Err(ValidationError::InvalidSlug { field: "order_by" }),
            }
        } else {
            return Err(ValidationError::InvalidSlug { field: "order_by" });
        };

        let limit = self.limit.unwrap_or(10).clamp(1, 100);
        let cursor = match self.cursor {
            None => None,
            Some(raw) => Some(decode_cursor(&raw)?),
        };

        let mut select = self
            .select
            .into_iter()
            .map(|s| SelectField::parse(&s))
            .collect::<ValidationResult<Vec<_>>>()?;
        if select.is_empty() {
            select = vec![
                SelectField::Id,
                SelectField::Type,
                SelectField::Status,
                SelectField::Created,
                SelectField::Modified,
            ];
        }

        let mut applied_where = self.where_;
        if !applied_where.contains_key("status") {
            if let Some(statuses) = &statuses {
                applied_where.insert(
                    "status".to_string(),
                    Value::Array(statuses.iter().cloned().map(Value::String).collect()),
                );
            }
        }

        Ok((
            DocumentQuery {
                query: self.query,
                types,
                statuses,
                archived,
                deleted,
                created_gte,
                created_lte,
                modified_gte,
                modified_lte,
                order,
                limit,
                cursor,
            },
            select,
            applied_where,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectField {
    Id,
    Type,
    Status,
    Created,
    Modified,
    Archived,
    Deleted,
    CurrentRevisionId,
    Content,
    Extensions,
    ExtensionKey(String),
    Title,
    Summary,
}

impl SelectField {
    pub fn parse(s: &str) -> ValidationResult<Self> {
        match s {
            "id" => Ok(Self::Id),
            "type" => Ok(Self::Type),
            "status" => Ok(Self::Status),
            "created" => Ok(Self::Created),
            "modified" => Ok(Self::Modified),
            "archived" => Ok(Self::Archived),
            "deleted" => Ok(Self::Deleted),
            "current_revision_id" => Ok(Self::CurrentRevisionId),
            "content" => Ok(Self::Content),
            "extensions" => Ok(Self::Extensions),
            "title" => Ok(Self::Title),
            "summary" => Ok(Self::Summary),
            _ if s.starts_with("extensions.") => {
                let key = s.trim_start_matches("extensions.");
                validate_slug("select.extensions", key)?;
                Ok(Self::ExtensionKey(key.to_string()))
            }
            _ => Err(ValidationError::InvalidSlug { field: "select" }),
        }
    }
}

pub fn project_rows(docs: &[Document], select: &[SelectField]) -> Vec<Map<String, Value>> {
    docs.iter()
        .map(|d| {
            let mut row = Map::new();
            for f in select {
                match f {
                    SelectField::Id => {
                        row.insert("id".to_string(), Value::String(d.id.to_string()));
                    }
                    SelectField::Type => {
                        row.insert(
                            "type".to_string(),
                            Value::String(d.doc_type.as_str().to_string()),
                        );
                    }
                    SelectField::Status => {
                        row.insert(
                            "status".to_string(),
                            Value::String(d.status.as_str().to_string()),
                        );
                    }
                    SelectField::Created => {
                        row.insert(
                            "created".to_string(),
                            Value::String(d.created_at.to_rfc3339()),
                        );
                    }
                    SelectField::Modified => {
                        row.insert(
                            "modified".to_string(),
                            Value::String(d.modified_at.to_rfc3339()),
                        );
                    }
                    SelectField::Archived => {
                        row.insert(
                            "archived".to_string(),
                            d.archived_at
                                .map(|t| Value::String(t.to_rfc3339()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    SelectField::Deleted => {
                        row.insert(
                            "deleted".to_string(),
                            d.deleted_at
                                .map(|t| Value::String(t.to_rfc3339()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    SelectField::CurrentRevisionId => {
                        row.insert(
                            "current_revision_id".to_string(),
                            d.current_revision_id
                                .map(|id| Value::String(id.to_string()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    SelectField::Content => {
                        row.insert("content".to_string(), d.content.clone());
                    }
                    SelectField::Extensions => {
                        row.insert(
                            "extensions".to_string(),
                            Value::Object(d.extensions.clone()),
                        );
                    }
                    SelectField::ExtensionKey(k) => {
                        row.insert(
                            format!("extensions.{k}"),
                            d.extensions.get(k).cloned().unwrap_or(Value::Null),
                        );
                    }
                    SelectField::Title => {
                        row.insert(
                            "title".to_string(),
                            d.extensions.get("title").cloned().unwrap_or(Value::Null),
                        );
                    }
                    SelectField::Summary => {
                        row.insert(
                            "summary".to_string(),
                            d.extensions.get("summary").cloned().unwrap_or(Value::Null),
                        );
                    }
                }
            }
            row
        })
        .collect()
}

pub fn encode_cursor(c: &DocumentQueryCursor) -> String {
    let json = serde_json::to_vec(c).expect("cursor serialization is infallible");
    general_purpose::STANDARD.encode(json)
}

pub fn decode_cursor(raw: &str) -> ValidationResult<DocumentQueryCursor> {
    let bytes = general_purpose::STANDARD
        .decode(raw)
        .map_err(|_| ValidationError::InvalidSlug { field: "cursor" })?;
    serde_json::from_slice(&bytes).map_err(|_| ValidationError::InvalidSlug { field: "cursor" })
}

fn parse_bool(field: &'static str, v: &Value) -> ValidationResult<bool> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(ValidationError::InvalidSlug { field }),
    }
}

fn parse_string_or_array(field: &'static str, v: &Value) -> ValidationResult<Vec<String>> {
    match v {
        Value::String(s) => Ok(vec![s.clone()]),
        Value::Array(a) => {
            let mut out = Vec::with_capacity(a.len());
            for it in a {
                match it {
                    Value::String(s) => out.push(s.clone()),
                    _ => return Err(ValidationError::InvalidSlug { field }),
                }
            }
            Ok(out)
        }
        _ => Err(ValidationError::InvalidSlug { field }),
    }
}

fn parse_rfc3339(field: &'static str, v: &Value) -> ValidationResult<DateTime<Utc>> {
    let s = match v {
        Value::String(s) => s,
        _ => return Err(ValidationError::InvalidSlug { field }),
    };
    let dt = DateTime::parse_from_rfc3339(s)
        .map_err(|_| ValidationError::InvalidSlug { field })?
        .with_timezone(&Utc);
    Ok(dt)
}
