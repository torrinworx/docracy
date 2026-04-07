use assert_cmd::prelude::*;
use serde_json::Value;
use std::process::Command;
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DOCRACY_TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
}

fn workspace_create_command() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("docracy"));
    let Some(database_url) = database_url() else {
        return cmd;
    };

    cmd.env("DATABASE_URL", database_url);
    cmd
}

#[test]
fn workspace_create_generates_a_uuid_by_default() {
    let Some(_) = database_url() else {
        return;
    };

    let assert = workspace_create_command()
        .args(["workspace", "create"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout is utf-8");
    let value: Value =
        serde_json::from_str(stdout.trim()).expect("workspace create stdout is json");
    let workspace_id = value
        .get("workspace_id")
        .and_then(Value::as_str)
        .expect("workspace_id string");

    let parsed = Uuid::parse_str(workspace_id).expect("workspace_id is uuid");
    assert_ne!(parsed, Uuid::nil());
}

#[test]
fn workspace_create_accepts_an_explicit_uuid() {
    let Some(_) = database_url() else {
        return;
    };

    let workspace_id = Uuid::from_u128(0x1234);
    let assert = workspace_create_command()
        .args([
            "workspace",
            "create",
            "--workspace-id",
            &workspace_id.to_string(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout is utf-8");
    let value: Value =
        serde_json::from_str(stdout.trim()).expect("workspace create stdout is json");
    assert_eq!(value["workspace_id"], workspace_id.to_string());
}

#[test]
fn workspace_create_rejects_nil_uuid() {
    let assert = workspace_create_command()
        .args([
            "workspace",
            "create",
            "--workspace-id",
            Uuid::nil().to_string().as_str(),
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr is utf-8");
    assert!(stderr.contains("validation_error"));
    assert!(stderr.contains("workspace_id"));
}
