use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn small_skill_content() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\nSome content.\n".to_string()
}

fn large_skill_content() -> String {
    (1..=200)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// === Existing tests (default text format, backward compatibility) ===

#[test]
fn exits_success_when_no_skill_files_found() {
    let dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path()).assert().success();
}

#[test]
fn exits_success_for_valid_skill_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path()).assert().success();
}

#[test]
fn exits_failure_for_skill_file_exceeding_line_limit() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("line_count"))
        .stdout(predicate::str::contains("SKILL.md"));
}

#[test]
fn reports_only_violating_files_when_mixed() {
    let dir = TempDir::new().unwrap();

    let good_dir = dir.path().join("good");
    fs::create_dir(&good_dir).unwrap();
    fs::write(good_dir.join("SKILL.md"), small_skill_content()).unwrap();

    let bad_dir = dir.path().join("bad");
    fs::create_dir(&bad_dir).unwrap();
    fs::write(bad_dir.join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    let output = cmd.current_dir(dir.path()).assert().failure();

    output
        .stdout(
            predicate::str::contains("bad/SKILL.md").or(predicate::str::contains("bad\\SKILL.md")),
        )
        .stdout(predicate::str::contains("good/SKILL.md").not());
}

#[test]
fn discovers_skill_files_in_nested_directories() {
    let dir = TempDir::new().unwrap();

    let nested = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("SKILL.md"));
}

// === Directory path argument tests ===

#[test]
fn accepts_directory_path_argument() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg(dir.path()).assert().success();
}

#[test]
fn accepts_directory_path_with_violations() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("line_count"))
        .stdout(predicate::str::contains("SKILL.md"));
}

#[test]
fn reports_error_for_nonexistent_directory() {
    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn reports_error_for_file_instead_of_directory() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("somefile.txt");
    fs::write(&file_path, "not a directory").unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a directory"));
}

// === --format text tests ===

#[test]
fn format_text_produces_same_output_as_default() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut default_cmd = cargo_bin_cmd!("nori-lint");
    let default_output = default_cmd.current_dir(dir.path()).output().unwrap();

    let mut text_cmd = cargo_bin_cmd!("nori-lint");
    let text_output = text_cmd
        .arg("--format")
        .arg("text")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert_eq!(default_output.stdout, text_output.stdout);
    assert_eq!(default_output.status.code(), text_output.status.code());
}

// === --format json tests ===

#[test]
fn format_json_outputs_valid_json_with_violation() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    let output = cmd
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");

    let arr = parsed.as_array().expect("output should be a JSON array");
    assert_eq!(arr.len(), 1);

    let diag = &arr[0];
    assert_eq!(diag["rule"], "line_count");
    assert_eq!(diag["file"], "SKILL.md");
    assert!(diag["message"].as_str().unwrap().contains("200"));
    assert!(diag["message"].as_str().unwrap().contains("150"));
    assert!(diag["line"].is_null());
    assert!(diag["snippet"].is_null());
}

#[test]
fn format_json_outputs_empty_array_when_no_violations() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    let output = cmd
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");

    let arr = parsed.as_array().expect("output should be a JSON array");
    assert!(arr.is_empty());
}

#[test]
fn format_json_outputs_multiple_diagnostics() {
    let dir = TempDir::new().unwrap();

    let dir_a = dir.path().join("a");
    fs::create_dir(&dir_a).unwrap();
    fs::write(dir_a.join("SKILL.md"), large_skill_content()).unwrap();

    let dir_b = dir.path().join("b");
    fs::create_dir(&dir_b).unwrap();
    fs::write(dir_b.join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    let output = cmd
        .arg("--format")
        .arg("json")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");

    let arr = parsed.as_array().expect("output should be a JSON array");
    assert_eq!(arr.len(), 2);

    // Both should be line_count violations
    for diag in arr {
        assert_eq!(diag["rule"], "line_count");
        assert!(diag["file"].as_str().unwrap().contains("SKILL.md"));
    }
}

#[test]
fn format_invalid_value_prints_error() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--format")
        .arg("xml")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("format"));
}

#[test]
fn format_missing_value_prints_error() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--format")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--format requires a value"));
}

#[test]
fn format_equals_syntax_works() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    let output = cmd
        .arg("--format=json")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--format=json should produce valid JSON");

    let arr = parsed.as_array().expect("output should be a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rule"], "line_count");
}
