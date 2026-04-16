use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_moonbit_duplicate_detection() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.mbt");

    let content = r#"
pub fn process_items(items : Array[Int]) -> Array[Int] {
  let result : Array[Int] = []
  let mut i = 0
  while i < items.length() {
    if items[i] > 0 {
      result.push(items[i] * 2)
    }
    i = i + 1
  }
  result
}

pub fn handle_items(data : Array[Int]) -> Array[Int] {
  let output : Array[Int] = []
  let mut j = 0
  while j < data.length() {
    if data[j] > 0 {
      output.push(data[j] * 2)
    }
    j = j + 1
  }
  output
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-moonbit")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.8")
        .assert()
        .success()
        .stdout(predicate::str::contains("process_items"))
        .stdout(predicate::str::contains("handle_items"));
}

#[test]
fn test_no_duplicates() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("unique.mbt");

    let content = r#"
pub fn fib(n : Int) -> Int {
  if n <= 1 {
    n
  } else {
    fib(n - 1) + fib(n - 2)
  }
}

pub fn sort_array(arr : Array[Int]) -> Array[Int] {
  let result = arr
  let mut i = 0
  while i < result.length() {
    let mut j = i + 1
    while j < result.length() {
      if result[j] < result[i] {
        let tmp = result[i]
        result[i] = result[j]
        result[j] = tmp
      }
      j = j + 1
    }
    i = i + 1
  }
  result
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-moonbit")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.8")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}

#[test]
fn test_threshold_filtering() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("threshold_test.mbt");

    let content = r#"
fn func1(x : Int) -> Int {
  let result = x + 1
  let doubled = result * 2
  doubled + 10
}

fn func2(y : Int) -> String {
  let temp = y.to_string()
  "value: " + temp
}
"#;

    fs::write(&file_path, content).unwrap();

    // With high threshold, should not detect as duplicate
    Command::cargo_bin("similarity-moonbit")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.95")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}

#[test]
fn test_min_lines_filtering() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("min_lines_test.mbt");

    let content = r#"
fn f1(x : Int) -> Int { x + 1 }

fn f2(x : Int) -> Int { x + 1 }

fn longer_func1(x : Int) -> Int {
  let a = x + 1
  let b = a * 2
  let c = b + 3
  c
}

fn longer_func2(y : Int) -> Int {
  let a = y + 1
  let b = a * 2
  let c = b + 3
  c
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-moonbit")
        .unwrap()
        .arg(&file_path)
        .arg("--min-lines")
        .arg("4")
        .assert()
        .success()
        .stdout(predicate::str::contains("longer_func1"))
        .stdout(predicate::str::contains("longer_func2"))
        .stdout(predicate::str::contains("f1").not());
}

#[test]
fn test_fail_on_duplicates() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("fail_test.mbt");

    let content = r#"
pub fn process_items(items : Array[Int]) -> Array[Int] {
  let result : Array[Int] = []
  let mut i = 0
  while i < items.length() {
    if items[i] > 0 {
      result.push(items[i] * 2)
    }
    i = i + 1
  }
  result
}

pub fn handle_items(data : Array[Int]) -> Array[Int] {
  let output : Array[Int] = []
  let mut j = 0
  while j < data.length() {
    if data[j] > 0 {
      output.push(data[j] * 2)
    }
    j = j + 1
  }
  output
}
"#;

    fs::write(&file_path, content).unwrap();

    Command::cargo_bin("similarity-moonbit")
        .unwrap()
        .arg(&file_path)
        .arg("--threshold")
        .arg("0.8")
        .arg("--fail-on-duplicates")
        .assert()
        .code(1);
}
