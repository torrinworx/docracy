use rmcp::ServiceExt;
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn tools_list_is_stable() {
    // rmcp is compiled with its `local` feature in this workspace, so it may use `spawn_local`.
    // Run the handshake inside a LocalSet to avoid runtime panics.
    tokio::task::LocalSet::new()
        .run_until(async {
            // Tool schemas can be sizeable; ensure the in-memory transport buffer is large enough.
            let (a, b) = tokio::io::duplex(4 * 1024 * 1024);

            // IMPORTANT: `serve(...)` performs the MCP initialization handshake and will wait for the
            // peer's initialize request/response. Start server+client concurrently to avoid deadlock.
            let (server, client) = timeout(Duration::from_secs(2), async move {
                let server = docracy_mcp::DocracyMcpServer::new_unbootstrapped();
                tokio::join!(server.serve(a), ().serve(b))
            })
            .await
            .expect("server/client initialization timed out");

            let mut server = server.expect("server initialization failed");
            let client = client.expect("client initialization failed");

            let tools = timeout(Duration::from_secs(2), client.list_all_tools())
                .await
                .expect("list_all_tools timed out")
                .unwrap();

            let mut names: Vec<String> = tools.into_iter().map(|t| t.name.to_string()).collect();
            names.sort();

            assert_eq!(names, vec![
                "create",
                "init",
                "query",
                "query_vector",
                "read",
                "update",
            ]);

            // Best-effort cleanup; don't let shutdown hang the test suite.
            let _ = server.close_with_timeout(Duration::from_millis(250)).await;
        })
        .await;
}

#[tokio::test]
async fn tool_input_schema_has_expected_properties() {
    tokio::task::LocalSet::new()
        .run_until(async {
            // Tool schemas can be sizeable; ensure the in-memory transport buffer is large enough.
            let (a, b) = tokio::io::duplex(4 * 1024 * 1024);

            let (server, client) = timeout(Duration::from_secs(2), async move {
                let server = docracy_mcp::DocracyMcpServer::new_unbootstrapped();
                tokio::join!(server.serve(a), ().serve(b))
            })
            .await
            .expect("server/client initialization timed out");

            let mut server = server.expect("server initialization failed");
            let client = client.expect("client initialization failed");

            let tools = timeout(Duration::from_secs(2), client.list_all_tools())
                .await
                .expect("list_all_tools timed out")
                .unwrap();
            let tools: HashMap<String, rmcp::model::Tool> =
                tools.into_iter().map(|t| (t.name.to_string(), t)).collect();

            let create = tools.get("create").expect("missing create tool");
            let create_schema = create.schema_as_json_value();
            let create_props = create_schema
                .get("properties")
                .and_then(Value::as_object)
                .expect("create schema properties should be an object");
            assert!(create_props.contains_key("type"));
            assert!(create_props.contains_key("content"));
            assert!(create_props.contains_key("extensions"));

            let update = tools.get("update").expect("missing update tool");
            let update_schema = update.schema_as_json_value();
            let update_props = update_schema
                .get("properties")
                .and_then(Value::as_object)
                .expect("update schema properties should be an object");
            assert!(update_props.contains_key("id"));
            assert!(update_props.contains_key("expected_revision"));

            // `expected_head` should be accepted as an alias for `expected_revision`.
            let args: docracy_mcp::tools::UpdateArgs = serde_json::from_value(serde_json::json!({
                "id": "00000000-0000-0000-0000-000000000000",
                "expected_head": "00000000-0000-0000-0000-000000000001"
            }))
            .unwrap();
            assert_eq!(
                args.expected_revision,
                "00000000-0000-0000-0000-000000000001".to_string()
            );

            let _ = server.close_with_timeout(Duration::from_millis(250)).await;
        })
        .await;
}

#[test]
fn error_data_contains_kind_and_optional_details() {
    let err = docracy_mcp::McpError::new(docracy_mcp::McpErrorKind::ValidationError, "bad input");
    let error_data = err.to_error_data();
    let data = error_data.data.expect("expected ErrorData.data to exist");
    let obj = data
        .as_object()
        .expect("expected ErrorData.data to be a JSON object");
    assert!(obj.contains_key("kind"));
    assert!(obj.contains_key("details"));
}
