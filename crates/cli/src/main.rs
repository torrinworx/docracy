#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use docracy_core::document::NewDocument;
use docracy_core::governance::FsGovernanceSource;
use docracy_core::ids::{DocumentId, RevisionId};
use docracy_core::query::QueryInput;
use docracy_core::service::{SystemClock, UuidV4Generator};
use docracy_core::{
    create_document, init_bundle, query_documents, read_documents, update_document,
    UpdateDocumentInput,
};
use docracy_postgres::PgRepository;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::PathBuf;

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
    /// Load governance docs + active context docs. Bootstraps constitution if missing.
    Init {
        /// Governance directory (default: ./governance)
        #[arg(long, default_value = "./governance")]
        governance_dir: PathBuf,
    },

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

    /// Apply migrations to the database
    Migrate,
}

#[derive(Debug, Deserialize)]
struct ReadInput {
    ids: Vec<DocumentId>,
}

#[derive(Debug, Deserialize)]
struct UpdateInput {
    id: DocumentId,
    expected_head: RevisionId,
    content: Option<Value>,
    extensions: Option<Map<String, Value>>,
    status: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        let out = json!({
            "error": err.to_string(),
        });
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string())
        );
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let database_url = cli
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow!("missing --database-url and DATABASE_URL"))?;

    let mut repo = PgRepository::connect(&database_url)
        .await
        .context("failed to connect to postgres")?;
    if !cli.no_migrate {
        repo.migrate().await.context("failed to run migrations")?;
    }

    let clock = SystemClock;
    let ids = UuidV4Generator;

    let output = match cli.command {
        Command::Migrate => json!({"ok": true}),

        Command::Init { governance_dir } => {
            let governance = FsGovernanceSource::new(governance_dir);
            let out = init_bundle(&mut repo, &governance, &clock, &ids).await?;

            let governance_files: Vec<Value> = out
                .governance
                .files
                .into_iter()
                .map(|f| json!({"name": f.name, "content": f.content}))
                .collect();

            json!({
                "governance": {"files": governance_files},
                "context_documents": out.context_documents,
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
                    expected_head: u.expected_head,
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
