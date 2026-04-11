use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_functions_within_file() {
    let dir = tempdir().unwrap();
    let sample_path = dir.path().join("sample.ts");

    // Create a file with similar functions
    fs::write(
        &sample_path,
        r#"
export function calculateSum(numbers: number[]): number {
    if (numbers.length === 0) return 0;
    
    let total = 0;
    for (const num of numbers) {
        total += num;
    }
    
    return total;
}

export function computeTotal(values: number[]): number {
    if (values.length === 0) return 0;
    
    let sum = 0;
    for (const val of values) {
        sum += val;
    }
    
    return sum;
}
"#,
    )
    .unwrap();

    // Run the CLI with no size penalty since these are short functions
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("-t")
        .arg("0.8")
        .arg("--no-size-penalty")
        .assert()
        .success()
        .stdout(predicate::str::contains("calculateSum"))
        .stdout(predicate::str::contains("computeTotal"))
        .stdout(predicate::str::contains("Similarity:"));
}

#[test]
fn test_functions_cross_file() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("file1.ts");
    let file2 = dir.path().join("file2.ts");

    // Create first file
    fs::write(
        &file1,
        r#"
export function processData(items: any[]): number {
    let result = 0;
    // Process each item
    for (const item of items) {
        result += item.value;
    }
    // Return the result
    return result;
}
"#,
    )
    .unwrap();

    // Create second file with similar function
    fs::write(
        &file2,
        r#"
export function calculateTotal(elements: any[]): number {
    let total = 0;
    // Process each element
    for (const element of elements) {
        total += element.value;
    }
    // Return the total
    return total;
}
"#,
    )
    .unwrap();

    // Run the CLI with lower threshold
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("-t")
        .arg("0.6")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates"));
}

#[test]
fn test_types_similarity() {
    let dir = tempdir().unwrap();
    let types_file = dir.path().join("types.ts");

    // Create a file with similar types
    fs::write(
        &types_file,
        r#"
interface User {
    id: string;
    name: string;
    email: string;
}

interface Person {
    id: string;
    name: string;
    email: string;
}

type UserData = {
    id: string;
    name: string;
    email: string;
};
"#,
    )
    .unwrap();

    // Run the CLI for types
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("--no-functions")
        .assert()
        .success()
        .stdout(predicate::str::contains("User"))
        .stdout(predicate::str::contains("Person"))
        .stdout(predicate::str::contains("UserData"))
        .stdout(predicate::str::contains("similar-type"));
}

#[test]
fn test_default_command_runs_functions_and_types() {
    let dir = tempdir().unwrap();
    let test_file = dir.path().join("test.ts");

    // Create a file with both functions and types
    fs::write(
        &test_file,
        r#"
// Similar functions
export function add(a: number, b: number): number {
    // Add two numbers together
    const result = a + b;
    // Return the result
    return result;
}

export function sum(x: number, y: number): number {
    // Sum two numbers together  
    const result = x + y;
    // Return the result
    return result;
}

// Similar types
interface IUser {
    name: string;
    age: number;
}

interface IPerson {
    name: string;
    age: number;
}
"#,
    )
    .unwrap();

    // Run default (functions + types)
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("--min-lines")
        .arg("3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Function Similarity"))
        .stdout(predicate::str::contains("Checking 1 files for duplicates"))
        .stdout(predicate::str::contains("Type Similarity"))
        .stdout(predicate::str::contains("IUser"))
        .stdout(predicate::str::contains("IPerson"));
}

#[test]
fn test_threshold_option() {
    let dir = tempdir().unwrap();
    let sample_path = dir.path().join("sample.ts");

    // Create functions with moderate similarity
    fs::write(
        &sample_path,
        r#"
export function processArray(arr: number[]): number {
    let result = 0;
    // Process each element
    for (let i = 0; i < arr.length; i++) {
        result += arr[i] * 2;
    }
    // Return the result
    return result;
}

export function handleList(list: number[]): number {
    let output = 0;
    // Process each element
    for (let j = 0; j < list.length; j++) {
        output += list[j] * 3;
    }
    // Return the output
    return output;
}
"#,
    )
    .unwrap();

    // With low threshold and no size penalty - should find similarity
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("--threshold")
        .arg("0.5")
        .arg("--no-size-penalty")
        .assert()
        .success()
        .stdout(predicate::str::contains("processArray"))
        .stdout(predicate::str::contains("handleList"));

    // With high threshold - should not find similarity
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .arg("--threshold")
        .arg("0.9")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found"));
}

#[test]
fn test_three_similar_functions_are_grouped_into_one_cluster() {
    let dir = tempdir().unwrap();
    let sample_path = dir.path().join("cluster.ts");

    fs::write(
        &sample_path,
        r#"
export function calculateSum(numbers: number[]): number {
    let total = 0;
    for (const number of numbers) {
        total += number;
    }
    return total;
}

export function computeTotal(values: number[]): number {
    let total = 0;
    for (const value of values) {
        total += value;
    }
    return total;
}

export function aggregateAmount(items: number[]): number {
    let total = 0;
    for (const item of items) {
        total += item;
    }
    return total;
}
"#,
    )
    .unwrap();

    Command::cargo_bin("similarity-ts")
        .unwrap()
        .arg(dir.path())
        .arg("--threshold")
        .arg("0.8")
        .arg("--min-lines")
        .arg("1")
        .arg("--no-size-penalty")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1 duplicate cluster"))
        .stdout(predicate::str::contains("calculateSum"))
        .stdout(predicate::str::contains("computeTotal"))
        .stdout(predicate::str::contains("aggregateAmount"));
}

#[test]
fn test_multiple_paths() {
    let dir = tempdir().unwrap();
    let dir1 = dir.path().join("src");
    let dir2 = dir.path().join("lib");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Create files in different directories
    fs::write(
        dir1.join("utils.ts"),
        r#"
export function double(n: number): number {
    // Double the input
    const result = n * 2;
    // Return the result
    return result;
}
"#,
    )
    .unwrap();

    fs::write(
        dir2.join("helpers.ts"),
        r#"
export function twice(num: number): number {
    // Multiply by two
    const result = num * 2;
    // Return the result  
    return result;
}
"#,
    )
    .unwrap();

    // Run with multiple paths
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(&dir1)
        .arg(&dir2)
        .arg("-t")
        .arg("0.6")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates"));
}

#[test]
fn test_ignores_node_modules() {
    let dir = tempdir().unwrap();
    let node_modules = dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();

    // Create files in node_modules (should be ignored)
    fs::write(
        node_modules.join("test.ts"),
        r#"
export function ignored() {
    return "This should be ignored";
}
"#,
    )
    .unwrap();

    // Create file in root (should be found)
    fs::write(
        dir.path().join("app.ts"),
        r#"
export function found() {
    return "This should be found";
}
"#,
    )
    .unwrap();

    // Run the CLI
    let mut cmd = Command::cargo_bin("similarity-ts").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found"));
}
