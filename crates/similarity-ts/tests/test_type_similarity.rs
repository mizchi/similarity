#![allow(clippy::uninlined_format_args)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_detect_similar_interfaces() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("user.ts");
    let file2 = dir.path().join("person.ts");

    // Very similar interfaces with different names
    let content1 = r#"
interface User {
    id: number;
    name: string;
    email: string;
    createdAt: Date;
}

interface Admin {
    id: number;
    name: string;
    email: string;
    role: string;
}
"#;

    let content2 = r#"
interface Person {
    id: number;
    name: string;
    email: string;
    birthDate: Date;
}

interface Customer {
    id: number;
    name: string;
    email: string;
    purchaseHistory: string[];
}
"#;

    fs::write(&file1, content1).unwrap();
    fs::write(&file2, content2).unwrap();

    Command::cargo_bin("similarity-ts")
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
fn test_detect_similar_type_aliases() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("types.ts");

    let content = r#"
type UserData = {
    id: number;
    name: string;
    email: string;
    active: boolean;
};

type PersonData = {
    id: number;
    name: string;
    email: string;
    enabled: boolean;
};

type AccountInfo = {
    id: string;
    username: string;
    emailAddress: string;
    isActive: boolean;
};
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--types-only")
        .arg("--threshold")
        .arg("0.8")
        .assert()
        .success()
        .stdout(predicate::str::contains("UserData"))
        .stdout(predicate::str::contains("PersonData"));
}

#[test]
fn test_interface_vs_type_alias_similarity() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("mixed.ts");

    let content = r#"
interface IUser {
    id: number;
    name: string;
    email: string;
    permissions: string[];
}

type TUser = {
    id: number;
    name: string;
    email: string;
    permissions: string[];
};

// Should be detected as highly similar despite different kinds
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--allow-cross-kind")
        .arg("--threshold")
        .arg("0.9")
        .assert()
        .success()
        .stdout(predicate::str::contains("IUser"))
        .stdout(predicate::str::contains("TUser"))
        .stdout(predicate::str::contains("Similarity:"));
}

#[test]
fn test_nested_type_similarity() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("nested.ts");

    let content = r#"
interface Order {
    id: string;
    items: {
        productId: string;
        quantity: number;
        price: number;
    }[];
    customer: {
        id: string;
        name: string;
        email: string;
    };
    totalAmount: number;
}

interface Purchase {
    orderId: string;
    products: {
        id: string;
        count: number;
        cost: number;
    }[];
    buyer: {
        userId: string;
        fullName: string;
        contactEmail: string;
    };
    total: number;
}
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.6")
        .assert()
        .success()
        .stdout(predicate::str::contains("Order"))
        .stdout(predicate::str::contains("Purchase"));
}

#[test]
fn test_generic_type_similarity() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("generics.ts");

    let content = r#"
interface Response<T> {
    data: T;
    status: number;
    message: string;
}

interface ApiResult<T> {
    result: T;
    code: number;
    description: string;
}

interface ServerResponse<T> {
    payload: T;
    statusCode: number;
    error?: string;
}
"#;

    fs::write(&file, content).unwrap();

    Command::cargo_bin("similarity-ts")
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
fn test_no_false_positives_different_types() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("different.ts");

    let content = r#"
interface Point {
    x: number;
    y: number;
}

interface User {
    id: string;
    name: string;
    email: string;
}

interface Config {
    debug: boolean;
    timeout: number;
    retryCount: number;
}
"#;

    fs::write(&file, content).unwrap();

    // High threshold should not detect these as similar
    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--experimental-types")
        .arg("--no-functions")
        .arg("--threshold")
        .arg("0.9")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No similar types found")
                .or(predicate::str::contains("Total duplicate pairs found: 0")),
        );
}

#[test]
fn test_type_fingerprint_performance() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("many_types.ts");

    // Generate many types to test fingerprint performance
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            r#"
interface Type{} {{
    field1: string;
    field2: number;
    field3: boolean;
    field4: {{"nested": number}};
    field{}: any;
}}
"#,
            i, i
        ));
    }

    fs::write(&file, content).unwrap();

    // Should complete quickly due to fingerprint optimization
    Command::cargo_bin("similarity-ts")
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
