use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_similarity_toml_supplies_default_options() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("sample.rs");

    fs::write(
        dir.path().join("similarity.toml"),
        r#"
threshold = 0.7
min_tokens = 1
no_size_penalty = true
"#,
    )
    .unwrap();

    fs::write(
        &file,
        r#"
fn add_one(x: i32) -> i32 {
    let value = x + 1;
    value
}

fn increment(y: i32) -> i32 {
    let result = y + 1;
    result
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .current_dir(dir.path())
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("add_one"))
        .stdout(predicate::str::contains("increment"));
}

#[test]
fn test_cli_flags_override_similarity_toml() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("sample.rs");

    fs::write(
        dir.path().join("similarity.toml"),
        r#"
threshold = 0.7
min_tokens = 1
no_size_penalty = true
"#,
    )
    .unwrap();

    fs::write(
        &file,
        r#"
fn add_one(x: i32) -> i32 {
    let value = x + 1;
    value
}

fn increment(y: i32) -> i32 {
    let result = y + 1;
    result
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .current_dir(dir.path())
        .arg(".")
        .arg("--min-tokens")
        .arg("30")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}
