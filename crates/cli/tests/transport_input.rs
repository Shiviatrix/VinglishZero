#![cfg(unix)]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/accumulate.ving"
);
const DIAGNOSTIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/type-mismatch.json"
);

static NEXT_TEMPORARY_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "vz-cli-transport-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_vz")
}

fn run(arguments: &[&str]) -> Output {
    Command::new(binary()).args(arguments).output().unwrap()
}

fn run_with_compiler(arguments: &[&str], compiler: &Path) -> Output {
    Command::new(binary())
        .args(arguments)
        .env("VZ_VINGLISH_COMPILER", compiler)
        .output()
        .unwrap()
}

fn fake_compiler(directory: &TemporaryDirectory, name: &str, body: &str) -> PathBuf {
    let compiler = directory.path().join(name);
    fs::write(&compiler, format!("#!/bin/sh\n{body}\n")).unwrap();
    let mut permissions = fs::metadata(&compiler).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&compiler, permissions).unwrap();
    compiler
}

#[test]
fn existing_json_workflow_renders_the_deterministic_intent_explanation() {
    let output = run(&[
        "explain",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/accumulate-v1.json"
        ),
    ]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim_end(),
        include_str!("../../../tests/fixtures/accumulate-v1.intent.txt").trim_end()
    );
}

#[test]
fn help_is_actionable_and_unknown_commands_fail() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("vz explain <source>"));
    assert!(help.contains("vz cache stats"));

    let unknown = run(&["unknown-command"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8(unknown.stderr)
        .unwrap()
        .contains("unknown command"));
}

#[test]
fn source_workflow_invokes_the_compiler_and_matches_json_import() {
    let directory = TemporaryDirectory::new();
    let compiler = fake_compiler(
        &directory,
        "vng-success",
        &format!("cat '{}'", transport_fixture().display()),
    );

    let from_source = run_with_compiler(&["explain", SOURCE], &compiler);
    let from_json = run(&["explain", transport_fixture().to_str().unwrap()]);

    assert!(from_source.status.success());
    assert!(from_json.status.success());
    assert_eq!(from_source.stdout, from_json.stdout);
}

#[test]
fn diagnose_accepts_source_through_the_same_transport_path() {
    let directory = TemporaryDirectory::new();
    let compiler = fake_compiler(
        &directory,
        "vng-success",
        &format!("cat '{}'", transport_fixture().display()),
    );

    let from_source = run_with_compiler(&["diagnose", DIAGNOSTIC, SOURCE], &compiler);
    let from_json = run(&[
        "diagnose",
        DIAGNOSTIC,
        transport_fixture().to_str().unwrap(),
    ]);

    assert!(from_source.status.success());
    assert!(from_json.status.success());
    assert_eq!(from_source.stdout, from_json.stdout);
}

#[test]
fn reports_a_missing_compiler_without_panicking() {
    let output = Command::new(binary())
        .args(["explain", SOURCE])
        .env("VZ_VINGLISH_COMPILER", "/definitely/not/a/vng")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("was not found"));
}

#[test]
fn reports_compiler_failure_and_preserves_its_exit_code() {
    let directory = TemporaryDirectory::new();
    let compiler = fake_compiler(
        &directory,
        "vng-failure",
        "echo 'syntactic failure' >&2\nexit 17",
    );

    let output = run_with_compiler(&["explain", SOURCE], &compiler);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("exit code 17"));
    assert!(stderr.contains("syntactic failure"));
}

#[test]
fn invalid_json_is_rejected_by_the_existing_importer() {
    let directory = TemporaryDirectory::new();
    let invalid = directory.path().join("invalid.json");
    fs::write(&invalid, "{not json").unwrap();

    let output = run(&["explain", invalid.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("invalid Vinglish export JSON"));
}

#[test]
fn malformed_transport_is_rejected_by_the_existing_importer() {
    let directory = TemporaryDirectory::new();
    let malformed = directory.path().join("malformed.json");
    fs::write(
        &malformed,
        r#"{"format":"vinglish.semantic-export","version":1,"program":{}}"#,
    )
    .unwrap();

    let output = run(&["explain", malformed.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("invalid Vinglish export JSON"));
}

#[test]
fn unsupported_transport_versions_are_rejected_by_the_existing_importer() {
    let directory = TemporaryDirectory::new();
    let unsupported = directory.path().join("unsupported.json");
    fs::write(
        &unsupported,
        r#"{"format":"vinglish.semantic-export","version":99,"program":{"modules":[]}}"#,
    )
    .unwrap();

    let output = run(&["explain", unsupported.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("unsupported export version: 99"));
}

#[test]
fn unsupported_languages_fail_cleanly_after_registry_selection() {
    let directory = TemporaryDirectory::new();
    let go_source = directory.path().join("example.go");
    fs::write(&go_source, "package main\n").unwrap();

    let output = run(&["explain", go_source.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("Go frontend is unavailable"));
}

#[test]
fn invalid_python_source_returns_a_deterministic_frontend_error() {
    let directory = TemporaryDirectory::new();
    let python_source = directory.path().join("broken.py");
    fs::write(&python_source, "def incomplete(:\n").unwrap();

    let output = run(&["explain", python_source.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("invalid Python source"));
}

#[test]
fn source_and_json_imports_produce_identical_semantic_graphs() {
    // Equal CLI explanations establish that the independent source acquisition
    // path reaches the same deterministic importer and SemanticGraph.
    let directory = TemporaryDirectory::new();
    let compiler = fake_compiler(
        &directory,
        "vng-success",
        &format!("cat '{}'", transport_fixture().display()),
    );

    let from_source = run_with_compiler(&["explain", SOURCE], &compiler);
    let from_json = run(&["explain", transport_fixture().to_str().unwrap()]);

    assert_eq!(from_source.stdout, from_json.stdout);
}

fn transport_fixture() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/accumulate-v1.json"
    ))
}
