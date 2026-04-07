use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn migrate_without_database_url_emits_structured_stderr_json() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("docracy"));
    cmd.arg("migrate");
    cmd.env_remove("DATABASE_URL");

    let assert = cmd.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr is utf-8");

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/missing_database_url.stderr.json");
    let expected = fs::read_to_string(fixture).expect("read stderr fixture");

    assert_eq!(stderr.trim_end(), expected.trim_end());
}

#[test]
fn workspace_create_with_malformed_uuid_emits_structured_stderr_json() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("docracy"));
    cmd.args(["workspace", "create", "--workspace-id", "not-a-uuid"]);
    cmd.env_remove("DATABASE_URL");

    let assert = cmd.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr is utf-8");

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/create_workspace_invalid_id.stderr.json");
    let expected = fs::read_to_string(fixture).expect("read stderr fixture");

    assert_eq!(stderr.trim_end(), expected.trim_end());
}
