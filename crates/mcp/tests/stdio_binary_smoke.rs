use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn startup_failure_writes_nothing_to_stdout() {
    // No args: should fail due to missing database url.
    Command::cargo_bin("docracy-mcp")
        .unwrap()
        .env("WORKSPACE_ID", "550e8400-e29b-41d4-a716-446655440000")
        .assert()
        .failure()
        .stdout("")
        .stderr(
            predicate::str::contains("\"kind\":\"setup_error\"")
                .and(predicate::str::contains("\"error\"")),
        );
}
