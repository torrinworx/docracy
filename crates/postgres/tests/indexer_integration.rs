use docracy_postgres::indexer::{IndexerConfig, IndexerRuntime};

fn set_env(key: &str, value: &str) {
    unsafe {
        std::env::set_var(key, value);
    }
}

#[test]
fn indexer_config_reads_worker_env() {
    set_env("WORKSPACE_ID", "00000000-0000-0000-0000-000000000001");
    set_env("OLLAMA_URL", "http://127.0.0.1:11434");
    set_env("OLLAMA_EMBED_MODEL", "embeddinggemma");
    set_env("QDRANT_URL", "http://127.0.0.1:6333");
    set_env("POLL_INTERVAL_MS", "250");
    set_env("BATCH_SIZE", "8");

    let config = IndexerConfig::from_env().expect("config should parse");

    assert_eq!(
        config.workspace_id.to_string(),
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(config.ollama_url, "http://127.0.0.1:11434");
    assert_eq!(config.ollama_embed_model, "embeddinggemma");
    assert_eq!(config.qdrant_url, "http://127.0.0.1:6333");
    assert_eq!(config.poll_interval_ms, 250);
    assert_eq!(config.batch_size, 8);
}

#[test]
fn indexer_claim_sql_uses_skip_locked_and_workspace_binding() {
    let sql = IndexerRuntime::claim_pending_jobs_sql();

    assert!(sql.contains("FOR UPDATE SKIP LOCKED"));
    assert!(sql.contains("docracy.workspace_id"));
    assert!(sql.contains("available_at <= now()"));
}

#[test]
fn indexer_entrypoint_stays_text_only() {
    let stdout = IndexerRuntime::startup_banner();

    assert!(!stdout.trim_start().starts_with('{'));
    assert!(!stdout.trim_start().starts_with('['));
}
