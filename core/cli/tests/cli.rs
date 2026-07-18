//! End-to-end exit-code and output contract tests for the cn binary
//! (D-044.2: 0 ok, 1 failure, 2 usage; stubs name their gates).
//!
//! Tests spawn the real binary via `CARGO_BIN_EXE_cn` so the asserted exit
//! statuses are the actual process exit codes, not an in-process proxy.
//! All fixture data is synthetic (I1).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIXTURE_GROUP_ID: &str = "00000000-0000-0000-0000-000000000010";
const FIXTURE_MEMBER_VIEWER: &str = "person:00000000-0000-0000-0000-0000000003e9";

fn cn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cn"))
        .args(args)
        .output()
        .expect("cn binary runs")
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().expect("cn exits with a code")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf8 stderr")
}

fn temp_template(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cn-cli-tests-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(name);
    std::fs::write(&path, contents).expect("write temp template");
    path
}

#[test]
fn missing_subcommand_exits_usage() {
    let output = cn(&[]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("missing subcommand"));
}

#[test]
fn unknown_subcommand_exits_usage() {
    let output = cn(&["frobnicate"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("unknown subcommand 'frobnicate'"));
}

#[test]
fn help_flag_exits_ok_and_names_commands() {
    let output = cn(&["--help"]);
    assert_eq!(exit_code(&output), 0);
    let text = stdout(&output);
    for command in ["validate", "export", "ingest", "snapshot"] {
        assert!(text.contains(command), "help names {command}");
    }
}

#[test]
fn help_subcommand_exits_ok() {
    let output = cn(&["help"]);
    assert_eq!(exit_code(&output), 0);
    assert!(stdout(&output).contains("Exit codes"));
}

#[test]
fn ingest_stub_exits_usage_naming_g_rat() {
    let output = cn(&["ingest"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("G-RAT"));
}

#[test]
fn snapshot_stub_exits_usage_naming_phase_2_envelope() {
    let output = cn(&["snapshot"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("Phase 2 snapshot data envelope"));
}

#[test]
fn validate_missing_argument_exits_usage() {
    let output = cn(&["validate"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("usage: cn validate"));
}

#[test]
fn validate_fixture_template_passes_with_report() {
    let path = repo_path("fixtures/templates/research-network.template.json");
    let output = cn(&["validate", path.to_str().expect("utf8 path")]);
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("machine-readable report json");
    assert_eq!(report["errors"], serde_json::json!([]));
    assert!(report["warnings"].is_array(), "warnings are always visible");
}

#[test]
fn validate_missing_file_exits_failure() {
    let output = cn(&["validate", "no-such-template.json"]);
    assert_eq!(exit_code(&output), 1);
    assert!(stderr(&output).contains("cannot read"));
}

#[test]
fn validate_structural_parse_error_exits_failure() {
    let path = temp_template("not-a-template.json", r#"{ "not": "a template" }"#);
    let output = cn(&["validate", path.to_str().expect("utf8 path")]);
    assert_eq!(exit_code(&output), 1);
    assert!(stderr(&output).contains("invalid template json"));
}

#[test]
fn validate_semantic_errors_exit_failure_with_report() {
    let path = temp_template(
        "no-kinds.template.json",
        r#"{
            "schema_version": "0.1.0",
            "template_id": "no-kinds-check",
            "name": "No Kinds Check",
            "description": "Synthetic router-test template with zero kinds.",
            "kinds": [],
            "edge_kinds": []
        }"#,
    );
    let output = cn(&["validate", path.to_str().expect("utf8 path")]);
    assert_eq!(exit_code(&output), 1);
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("machine-readable report json");
    assert!(
        report["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|finding| finding["code"] == "NoKinds"),
        "report carries the NoKinds finding: {report}"
    );
}

#[test]
fn export_missing_required_argument_exits_usage() {
    let template = repo_path("fixtures/templates/research-network.template.json");
    let ops = repo_path("fixtures/groups/research-network.ops.jsonl");
    let output = cn(&[
        "export",
        "--template",
        template.to_str().expect("utf8 path"),
        "--ops",
        ops.to_str().expect("utf8 path"),
        "--group",
        FIXTURE_GROUP_ID,
    ]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("--viewer is required"));
}

#[test]
fn export_unknown_argument_exits_usage() {
    let output = cn(&["export", "--frobnicate", "value"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("unknown export argument"));
}

#[test]
fn export_rejects_unknown_viewer_scope() {
    let template = repo_path("fixtures/templates/research-network.template.json");
    let ops = repo_path("fixtures/groups/research-network.ops.jsonl");
    let output = cn(&[
        "export",
        "--template",
        template.to_str().expect("utf8 path"),
        "--ops",
        ops.to_str().expect("utf8 path"),
        "--group",
        FIXTURE_GROUP_ID,
        "--viewer",
        "wizard",
    ]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("viewer scope"));
}

#[test]
fn export_bad_group_id_exits_failure_naming_step() {
    let template = repo_path("fixtures/templates/research-network.template.json");
    let ops = repo_path("fixtures/groups/research-network.ops.jsonl");
    let output = cn(&[
        "export",
        "--template",
        template.to_str().expect("utf8 path"),
        "--ops",
        ops.to_str().expect("utf8 path"),
        "--group",
        "not-a-uuid",
        "--viewer",
        "anonymous",
    ]);
    assert_eq!(exit_code(&output), 1);
    assert!(stderr(&output).contains("load_group_begin failed"));
}

#[test]
fn export_fixture_anonymous_exits_ok_with_envelope() {
    let template = repo_path("fixtures/templates/research-network.template.json");
    let ops = repo_path("fixtures/groups/research-network.ops.jsonl");
    let output = cn(&[
        "export",
        "--template",
        template.to_str().expect("utf8 path"),
        "--ops",
        ops.to_str().expect("utf8 path"),
        "--group",
        FIXTURE_GROUP_ID,
        "--viewer",
        "anonymous",
    ]);
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("envelope json");
    assert!(envelope.get("ok").is_some(), "envelope: {envelope}");
    assert!(envelope["ok"]["projection"]["entities"].is_array());
}

#[test]
fn export_fixture_member_viewer_projects_entities() {
    let template = repo_path("fixtures/templates/research-network.template.json");
    let ops = repo_path("fixtures/groups/research-network.ops.jsonl");
    let output = cn(&[
        "export",
        "--template",
        template.to_str().expect("utf8 path"),
        "--ops",
        ops.to_str().expect("utf8 path"),
        "--group",
        FIXTURE_GROUP_ID,
        "--viewer",
        FIXTURE_MEMBER_VIEWER,
    ]);
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("envelope json");
    let entities = envelope["ok"]["projection"]["entities"]
        .as_array()
        .expect("entities array");
    assert!(
        !entities.is_empty(),
        "member viewer export projects entities"
    );
}
