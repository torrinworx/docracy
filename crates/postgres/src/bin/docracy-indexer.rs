use docracy_postgres::indexer::IndexerRuntime;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let runtime = IndexerRuntime::connect_from_env(&database_url).await?;

    eprintln!("{}", IndexerRuntime::startup_banner());
    runtime.run().await?;

    Ok(())
}
