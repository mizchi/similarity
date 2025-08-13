#![allow(clippy::uninlined_format_args)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_detect_similar_structs() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("user.rs");
    let file2 = dir.path().join("person.rs");

    // Very similar structs with different names
    let content1 = r#"
struct User {
    id: u64,
    name: String,
    email: String,
    created_at: SystemTime,
}

struct Admin {
    id: u64,
    name: String,
    email: String,
    role: String,
}
"#;

    let content2 = r#"
struct Person {
    id: u64,
    full_name: String,
    email_address: String,
    birth_date: SystemTime,
}

struct Customer {
    customer_id: u64,
    customer_name: String,
    contact_email: String,
    orders: Vec<Order>,
}
"#;

    fs::write(&file1, content1).unwrap();
    fs::write(&file2, content2).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.7")
        .assert()
        .success()
        .stdout(predicate::str::contains("User"))
        .stdout(predicate::str::contains("Person"))
        .stdout(predicate::str::contains("Similarity:"));
}

#[test]
fn test_detect_similar_enums() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("enums.rs");

    let content = r#"
enum Status {
    Active,
    Inactive,
    Pending,
    Completed,
}

enum State {
    Running,
    Stopped,
    Waiting,
    Finished,
}

enum TaskStatus {
    InProgress,
    Paused,
    Queued,
    Done,
}
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.7")
        .assert()
        .success()
        .stdout(predicate::str::contains("Status"))
        .stdout(predicate::str::contains("State"));
}

#[test]
fn test_struct_with_generics() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("generics.rs");

    let content = r#"
struct Response<T> {
    data: T,
    status: u16,
    message: String,
}

struct ApiResult<T> {
    result: T,
    code: u16,
    description: String,
}

struct ServerResponse<T> {
    payload: T,
    status_code: u16,
    error: Option<String>,
}
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.7")
        .assert()
        .success()
        .stdout(predicate::str::contains("Response"))
        .stdout(predicate::str::contains("ApiResult"));
}

#[test]
fn test_struct_fingerprint_performance() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("many_structs.rs");

    // Generate many structs to test fingerprint performance
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            r#"
struct Type{} {{
    field1: String,
    field2: u32,
    field3: bool,
    field4: HashMap<String, u32>,
    field{}: Vec<u8>,
}}
"#,
            i, i
        ));
    }

    fs::write(&file, content).unwrap();

    // Should complete quickly due to fingerprint optimization
    Command::cargo_bin("similarity-rs")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.95")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}