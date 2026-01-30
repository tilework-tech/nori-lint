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
