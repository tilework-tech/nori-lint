use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn small_skill_content() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome content.\n</required>\n"
        .to_string()
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
    let line_count_diag = arr
        .iter()
        .find(|d| d["rule"] == "line_count")
        .expect("should contain a line_count violation");
    assert_eq!(line_count_diag["file"], "SKILL.md");
    assert!(line_count_diag["message"].as_str().unwrap().contains("200"));
    assert!(line_count_diag["message"].as_str().unwrap().contains("150"));
    assert!(line_count_diag["line"].is_null());
    assert!(line_count_diag["snippet"].is_null());
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
    assert!(
        arr.len() >= 2,
        "should have at least 2 diagnostics across two files, got {}",
        arr.len()
    );

    // Each file should have at least one line_count violation
    let line_count_diags: Vec<_> = arr.iter().filter(|d| d["rule"] == "line_count").collect();
    assert_eq!(
        line_count_diags.len(),
        2,
        "should have exactly 2 line_count violations"
    );
    for diag in &line_count_diags {
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
        .stderr(predicate::str::contains("--format"));
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
    assert!(
        arr.iter().any(|d| d["rule"] == "line_count"),
        "should contain a line_count violation"
    );
}

// === required_tags rule tests ===

fn skill_without_required_tags() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\nSome content without required tags.\n"
        .to_string()
}

#[test]
fn exits_failure_for_skill_file_missing_required_tags() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_without_required_tags()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("required_tags"));
}

#[test]
fn format_json_includes_required_tags_violation() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_without_required_tags()).unwrap();

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
    let required_diag = arr
        .iter()
        .find(|d| d["rule"] == "required_tags")
        .expect("should contain a required_tags violation");
    assert_eq!(required_diag["file"], "SKILL.md");
    assert!(
        required_diag["message"]
            .as_str()
            .unwrap()
            .contains("required")
    );
}

// === --help tests ===

#[test]
fn help_flag_prints_usage_and_exits_success() {
    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("nori-lint"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("Rules:").not());
}

#[test]
fn short_help_flag_prints_usage_and_exits_success() {
    let mut cmd = cargo_bin_cmd!("nori-lint");
    let help_output = cargo_bin_cmd!("nori-lint").arg("--help").output().unwrap();

    let short_output = cmd.arg("-h").output().unwrap();

    assert_eq!(help_output.stdout, short_output.stdout);
    assert_eq!(help_output.status.code(), short_output.status.code());
}

#[test]
fn help_flag_takes_priority_over_other_arguments() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), large_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--help")
        .arg("--format")
        .arg("json")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

// === unclosed_tags rule tests ===

fn skill_with_unclosed_tag() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome content.\n"
        .to_string()
}

#[test]
fn exits_failure_for_skill_file_with_unclosed_tag() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_unclosed_tag()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("unclosed_tags"));
}

#[test]
fn format_json_includes_unclosed_tags_violation() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_unclosed_tag()).unwrap();

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
    let unclosed_diag = arr
        .iter()
        .find(|d| d["rule"] == "unclosed_tags")
        .expect("should contain an unclosed_tags violation");
    assert_eq!(unclosed_diag["file"], "SKILL.md");
    assert!(
        unclosed_diag["message"]
            .as_str()
            .unwrap()
            .contains("required")
    );
}

// === --config tests ===

#[test]
fn without_config_deterministic_rules_still_run() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();
    // No config.json present, no --config flag
    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "note: skipping LLM rules (no .nori-lint.json found; use --config to specify)",
        ));
}

#[test]
fn config_flag_with_nonexistent_file_prints_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--config")
        .arg("/nonexistent/config.json")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("config"));
}

#[test]
fn config_flag_with_invalid_json_prints_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();
    fs::write(dir.path().join("config.json"), "not valid json").unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--config")
        .arg(dir.path().join("config.json"))
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("config"));
}

#[test]
fn config_flag_with_missing_api_key_prints_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();
    fs::write(dir.path().join("config.json"), r#"{"some_field": "value"}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--config")
        .arg(dir.path().join("config.json"))
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("anthropic_api_key"));
}

#[test]
fn config_equals_syntax_works() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();
    fs::write(
        dir.path().join("myconfig.json"),
        r#"{"some_field": "value"}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg(format!(
        "--config={}",
        dir.path().join("myconfig.json").display()
    ))
    .current_dir(dir.path())
    .assert()
    .failure()
    .stderr(predicate::str::contains("anthropic_api_key"));
}

#[test]
fn config_flag_missing_value_prints_error() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.arg("--config")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

// === redundant_title rule tests ===

fn skill_with_redundant_title() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\n# The Test Skill\n\n<required>\nSome content.\n</required>\n"
        .to_string()
}

#[test]
fn exits_failure_for_skill_file_with_redundant_title() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_redundant_title()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("redundant_title"));
}

#[test]
fn format_json_includes_redundant_title_violation() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_redundant_title()).unwrap();

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
    let title_diag = arr
        .iter()
        .find(|d| d["rule"] == "redundant_title")
        .expect("should contain a redundant_title violation");
    assert_eq!(title_diag["file"], "SKILL.md");
    assert!(
        title_diag["line"].as_u64().is_some(),
        "should have a line number"
    );
    assert!(
        title_diag["snippet"].as_str().is_some(),
        "should have a snippet"
    );
}

#[test]
fn nori_lint_json_in_cwd_is_auto_discovered() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), small_skill_content()).unwrap();
    // Write an invalid config to prove it was loaded (missing api key -> error)
    fs::write(
        dir.path().join(".nori-lint.json"),
        r#"{"some_field": "value"}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("anthropic_api_key"));
}

// === bold_italics rule tests ===

fn skill_with_bold_text() -> String {
    "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome **bold** content.\n</required>\n"
        .to_string()
}

#[test]
fn exits_failure_for_skill_file_with_bold_text() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_bold_text()).unwrap();

    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("bold_italics"));
}

#[test]
fn format_json_includes_bold_italics_violation_with_line_number() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("SKILL.md"), skill_with_bold_text()).unwrap();

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
    let bold_diag = arr
        .iter()
        .find(|d| d["rule"] == "bold_italics")
        .expect("should contain a bold_italics violation");
    assert_eq!(bold_diag["file"], "SKILL.md");
    assert!(
        bold_diag["line"].as_u64().is_some(),
        "should have a line number"
    );
    assert!(
        bold_diag["snippet"].as_str().is_some(),
        "should have a snippet"
    );
}


