use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_similarity_ignore_skips_function_duplicates_and_reports_ignored_items() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("functions.ts");

    fs::write(
        &file,
        r#"
export function calculateTotal(items: number[]): number {
    let total = 0;
    for (const item of items) {
        total += item;
    }
    return total;
}

// similarity-ignore
export function computeTotal(values: number[]): number {
    let total = 0;
    for (const value of values) {
        total += value;
    }
    return total;
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--no-size-penalty")
        .arg("--show-ignored")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"))
        .stdout(predicate::str::contains("computeTotal"))
        .stdout(predicate::str::contains("Ignored"));
}

#[test]
fn test_similarity_ignore_skips_type_duplicates() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("types.ts");

    fs::write(
        &file,
        r#"
interface User {
    id: string;
    email: string;
}

// similarity-ignore
interface IgnoredUser {
    id: string;
    email: string;
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--no-functions")
        .arg("--show-ignored")
        .assert()
        .success()
        .stdout(predicate::str::contains("No similar types found"))
        .stdout(predicate::str::contains("IgnoredUser"));
}

#[test]
fn test_similarity_ignore_skips_class_duplicates() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("classes.ts");

    fs::write(
        &file,
        r#"
class ServiceA {
    process(input: string): string {
        return input.trim().toLowerCase();
    }
}

// similarity-ignore
class ServiceB {
    process(value: string): string {
        return value.trim().toLowerCase();
    }
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--classes-only")
        .arg("--show-ignored")
        .assert()
        .success()
        .stdout(predicate::str::contains("No similar classes found!"))
        .stdout(predicate::str::contains("ServiceB"));
}
