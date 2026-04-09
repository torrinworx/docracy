use clap::Parser;
use docracy_mcp::{McpStartupConfig, McpTransport};
use rmcp::ServiceExt;
use std::io::Write;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "docracy-mcp")]
#[command(about = "Docracy MCP server (stdio)")]
struct Args {
    /// Postgres connection string.
    /// If omitted, `DATABASE_URL` is used.
    #[arg(long)]
    database_url: Option<String>,

    /// Skip SQL migrations on startup.
    #[arg(long)]
    no_migrate: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // rmcp is built with its `local` feature here (core traits aren't Send/Sync). Ensure local
    // tasks are supported so stdio startup never panics.
    let res = tokio::task::LocalSet::new().run_until(run()).await;
    if let Err(err) = res {
        write_setup_error_and_exit(err);
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    // Stdio transport: stdout is reserved for protocol messages.
    // Logging must never go to stdout.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let workspace_id =
        docracy_mcp::config::parse_workspace_id(std::env::var("WORKSPACE_ID").ok().as_deref())
            .map_err(|err| anyhow::anyhow!("invalid WORKSPACE_ID: {err}"))?;
    let task_scope =
        docracy_mcp::config::parse_task_scope(std::env::var("DOCRACY_TASK_SCOPE").ok().as_deref());

    let database_url = match args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
    {
        Some(url) => url,
        None => anyhow::bail!("missing database url (pass --database-url or set DATABASE_URL)"),
    };

    let config = McpStartupConfig::new(
        database_url,
        !args.no_migrate,
        workspace_id,
        task_scope,
        McpTransport::Stdio,
    );

    let runtime = docracy_mcp::bootstrap(&config).await?;
    let service = docracy_mcp::DocracyMcpServer::new(runtime);

    let transport = rmcp::transport::stdio();
    let server = service.serve(transport).await?;

    // Block until the transport is closed or the service is cancelled.
    let _ = server.waiting().await?;
    Ok(())
}

fn write_setup_error_and_exit(err: anyhow::Error) -> ! {
    // Stdio safety: never write non-protocol output to stdout.
    let envelope = serde_json::json!({
        "error": {
            "kind": "setup_error",
            "message": err.to_string(),
        }
    });

    let _ = writeln!(std::io::stderr(), "{}", envelope);
    std::process::exit(1)
}
