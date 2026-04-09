#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use docracy_core::document::NewDocument;
use docracy_core::errors::{CoreError, GovernanceError, RepoError};
use docracy_core::governance::FsGovernanceSource;
use docracy_core::ids::{DocumentId, RevisionId};
use docracy_core::query::QueryInput;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{
    create_document, init_bundle_scoped, query_documents, read_documents, update_document,
    UpdateDocumentInput,
};
use docracy_postgres::PgRepository;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::io::Read;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct CliErrorResponse {
    error: CliErrorBody,
}

#[derive(Debug, Serialize)]
struct CliErrorBody {
    kind: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

#[derive(Parser, Debug)]
#[command(name = "docracy", version, about = "Docracy CLI")]
struct Cli {
    /// Postgres connection string (falls back to DATABASE_URL)
    #[arg(long)]
    database_url: Option<String>,

    /// Disable automatically running migrations
    #[arg(long)]
    no_migrate: bool,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Load governance docs + active context docs. Bootstraps the repo-owned governance doc if missing.
    Init {},

    /// Create a document (JSON input)
    Create {
        /// Input JSON file path (use '-' or omit to read from stdin)
        #[arg(long)]
        input: Option<String>,
    },

    /// Query documents (JSON input)
    Query {
        /// Input JSON file path (use '-' or omit to read from stdin)
        #[arg(long)]
        input: Option<String>,
    },

    /// Read documents by ids (JSON input)
    Read {
        /// Input JSON file path (use '-' or omit to read from stdin)
        #[arg(long)]
        input: Option<String>,
    },

    /// Update a document (JSON input)
    Update {
        /// Input JSON file path (use '-' or omit to read from stdin)
        #[arg(long)]
        input: Option<String>,
    },

    /// Workspace provisioning commands
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },

    /// Apply migrations to the database
    Migrate,
}

#[derive(Subcommand, Debug)]
enum WorkspaceCommand {
    /// Create a workspace row and return its UUID
    Create {
        /// Optional workspace UUID override for scripted provisioning
        #[arg(long)]
        workspace_id: Option<String>,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct CliValidationError {
    message: String,
}

impl CliValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReadInput {
    ids: Vec<DocumentId>,
}

#[derive(Debug, Deserialize)]
struct UpdateInput {
    id: DocumentId,
    #[serde(alias = "expected_head")]
    expected_revision: RevisionId,
    content: Option<Value>,
    extensions: Option<Map<String, Value>>,
    status: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        let out = cli_error_response(&err);
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|_| serde_json::to_string(&out).unwrap())
        );
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let workspace_create_id = match &cli.command {
        Command::Workspace {
            command: WorkspaceCommand::Create { workspace_id },
        } => Some(resolve_workspace_id(workspace_id.as_deref())?),
        _ => None,
    };

    let database_url = cli
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow!("missing --database-url and DATABASE_URL"))?;

    let mut repo = PgRepository::connect(&database_url)
        .await
        .context("failed to connect to postgres")?;
    if should_run_startup_migrations(&cli.command, cli.no_migrate) {
        repo.migrate().await.context("failed to run migrations")?;
    }

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let output = match cli.command {
        Command::Migrate => json!({"ok": true}),

        Command::Workspace { command } => match command {
            WorkspaceCommand::Create { .. } => {
                let workspace_id = workspace_create_id.expect("workspace id prevalidated");
                repo.create_workspace(workspace_id).await?;

                json!({"workspace_id": workspace_id.to_string()})
            }
        },

        Command::Init {} => {
            let governance = FsGovernanceSource::repo_owned();
            let task_scope = normalize_task_scope(std::env::var("DOCRACY_TASK_SCOPE").ok());
            let out = init_bundle_scoped(
                &mut repo,
                &governance,
                &clock,
                &ids,
                task_scope.as_deref(),
            )
            .await?;

            let governance_files: Vec<Value> = out
                .governance
                .files
                .into_iter()
                .map(|f| json!({"name": f.name, "content": f.content}))
                .collect();

            json!({
                "governance": {"files": governance_files},
                "context_documents": out.context_documents,
                "task_scope": out.task_scope,
                "task_context_documents": out.task_context_documents,
            })
        }

        Command::Create { input } => {
            let doc: NewDocument = read_json_input(input.as_deref())?;
            let out = create_document(&mut repo, &clock, &ids, doc).await?;
            json!({"document": out.document, "revision": out.revision})
        }

        Command::Query { input } => {
            let q: QueryInput = read_json_input(input.as_deref())?;
            let out = query_documents(&repo, q).await?;
            serde_json::to_value(out)?
        }

        Command::Read { input } => {
            let r: ReadInput = read_json_input(input.as_deref())?;
            let out = read_documents(&repo, &r.ids).await?;
            json!({"documents": out.documents, "missing_ids": out.missing_ids})
        }

        Command::Update { input } => {
            let u: UpdateInput = read_json_input(input.as_deref())?;
            let out = update_document(
                &mut repo,
                &clock,
                &ids,
                UpdateDocumentInput {
                    id: u.id,
                    expected_head: u.expected_revision,
                    content: u.content,
                    extensions: u.extensions,
                    status: u.status,
                },
            )
            .await?;

            json!({
                "document": out.document,
                "new_revision": out.new_revision,
                "superseded_revision": out.superseded_revision,
            })
        }
    };

    print_json(output, cli.pretty)?;
    Ok(())
}

fn should_run_startup_migrations(command: &Command, no_migrate: bool) -> bool {
    matches!(command, Command::Migrate) || !no_migrate
}

fn read_json_input<T: DeserializeOwned>(input: Option<&str>) -> Result<T> {
    let raw = match input {
        None | Some("-") => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .context("failed reading stdin")?;
            if s.trim().is_empty() {
                return Err(anyhow!("empty stdin; provide --input <file> or pipe JSON"));
            }
            s
        }
        Some(path) => std::fs::read_to_string(path).with_context(|| format!("read {path}"))?,
    };

    serde_json::from_str(&raw).context("invalid JSON")
}

fn print_json(v: Value, pretty: bool) -> Result<()> {
    let s = if pretty {
        serde_json::to_string_pretty(&v)?
    } else {
        serde_json::to_string(&v)?
    };
    println!("{s}");
    Ok(())
}

fn normalize_task_scope(raw: Option<String>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn resolve_workspace_id(input: Option<&str>) -> Result<Uuid> {
    match input {
        None => Ok(Uuid::new_v4()),
        Some(value) => {
            let workspace_id = Uuid::parse_str(value).map_err(|_| {
                CliValidationError::new(format!("workspace_id must be a UUID: {value}"))
            })?;

            if workspace_id.is_nil() {
                return Err(
                    CliValidationError::new("workspace_id must not be the nil UUID").into(),
                );
            }

            Ok(workspace_id)
        }
    }
}

fn cli_error_response(err: &anyhow::Error) -> CliErrorResponse {
    if let Some(validation_err) = err.downcast_ref::<CliValidationError>() {
        return CliErrorResponse {
            error: CliErrorBody {
                kind: "validation_error".to_string(),
                message: validation_err.to_string(),
                details: None,
            },
        };
    }

    if let Some(core_err) = err.downcast_ref::<CoreError>() {
        return CliErrorResponse {
            error: match core_err {
                CoreError::Validation(inner) => CliErrorBody {
                    kind: "validation_error".to_string(),
                    message: inner.to_string(),
                    details: None,
                },
                CoreError::Repo(RepoError::Conflict) => CliErrorBody {
                    kind: "conflict".to_string(),
                    message: "revision conflict".to_string(),
                    details: None,
                },
                CoreError::Repo(RepoError::WorkspaceNotProvisioned { workspace_id }) => {
                    CliErrorBody {
                        kind: "workspace_not_provisioned".to_string(),
                        message: core_err.to_string(),
                        details: Some(json!({
                            "workspace_id": workspace_id,
                        })),
                    }
                }
                CoreError::Repo(RepoError::Storage(message)) => CliErrorBody {
                    kind: "storage_error".to_string(),
                    message: message.clone(),
                    details: None,
                },
                CoreError::Governance(GovernanceError::Io(message)) => CliErrorBody {
                    kind: "governance_io_error".to_string(),
                    message: message.clone(),
                    details: None,
                },
                CoreError::Governance(GovernanceError::MissingGovernance) => CliErrorBody {
                    kind: "missing_governance".to_string(),
                    message: "missing governance instructions in governance bundle".to_string(),
                    details: None,
                },
                CoreError::DocumentNotFound => CliErrorBody {
                    kind: "document_not_found".to_string(),
                    message: core_err.to_string(),
                    details: None,
                },
                CoreError::RevisionNotFound => CliErrorBody {
                    kind: "revision_not_found".to_string(),
                    message: core_err.to_string(),
                    details: None,
                },
                CoreError::MissingCurrentRevision => CliErrorBody {
                    kind: "missing_current_revision".to_string(),
                    message: core_err.to_string(),
                    details: None,
                },
                CoreError::NoChanges => CliErrorBody {
                    kind: "no_changes".to_string(),
                    message: core_err.to_string(),
                    details: None,
                },
                CoreError::RevisionConflict { expected, actual } => CliErrorBody {
                    kind: "revision_conflict".to_string(),
                    message: core_err.to_string(),
                    details: Some(json!({
                        "expected": expected,
                        "actual": actual,
                    })),
                },
            },
        };
    }

    CliErrorResponse {
        error: CliErrorBody {
            kind: "internal_error".to_string(),
            message: err.to_string(),
            details: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use uuid::Uuid;

    #[test]
    fn migrate_subcommand_ignores_no_migrate_flag() {
        assert!(should_run_startup_migrations(&Command::Migrate, true));
    }

    #[test]
    fn ordinary_commands_still_honor_no_migrate_flag() {
        assert!(!should_run_startup_migrations(
            &Command::Query { input: None },
            true,
        ));
        assert!(should_run_startup_migrations(
            &Command::Query { input: None },
            false,
        ));
    }

    #[test]
    fn cli_parses_workspace_create_subcommand() {
        let cli = Cli::try_parse_from(["docracy", "workspace", "create"])
            .expect("workspace create parses");

        match cli.command {
            Command::Workspace {
                command: WorkspaceCommand::Create { workspace_id },
            } => {
                assert!(workspace_id.is_none());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_error_response_marks_validation_errors() {
        let err = anyhow!(CliValidationError::new("workspace_id must be a UUID"));
        let response = cli_error_response(&err);

        assert_eq!(response.error.kind, "validation_error");
        assert_eq!(response.error.message, "workspace_id must be a UUID");
    }

    #[test]
    fn resolve_workspace_id_generates_a_non_nil_uuid_by_default() {
        let resolved = resolve_workspace_id(None).expect("default workspace id");
        assert_ne!(resolved, Uuid::nil());
    }

    #[test]
    fn resolve_workspace_id_accepts_an_explicit_uuid() {
        let expected = Uuid::from_u128(0x1234);
        let resolved = resolve_workspace_id(Some(&expected.to_string())).expect("explicit id");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_workspace_id_rejects_nil_uuid() {
        let nil = Uuid::nil().to_string();
        let err = resolve_workspace_id(Some(&nil)).unwrap_err();
        assert!(err.to_string().contains("nil"));
    }

    #[test]
    fn normalize_task_scope_trims_whitespace() {
        assert_eq!(
            normalize_task_scope(Some("  planning  ".to_string())),
            Some("planning".to_string())
        );
    }

    #[test]
    fn normalize_task_scope_treats_whitespace_only_as_none() {
        assert_eq!(normalize_task_scope(Some("   ".to_string())), None);
    }

    #[test]
    fn normalize_task_scope_preserves_missing_values() {
        assert_eq!(normalize_task_scope(None), None);
    }
}
