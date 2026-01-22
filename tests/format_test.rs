use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn init_test_repo(dir: &TempDir) {
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("init")
        .assert()
        .success();
}

fn create_wire(dir: &TempDir, title: &str) -> String {
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("new")
        .arg(title)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["id"].as_str().unwrap().to_string()
}

// Test --format json is explicit and works
#[test]
fn test_list_format_json() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);
    create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    // Should be valid JSON array
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
}

#[test]
fn test_list_format_table() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);
    create_wire(&temp_dir, "Test wire");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .arg("--format")
        .arg("table")
        .assert()
        .success()
        .stdout(predicate::str::contains("ID"))
        .stdout(predicate::str::contains("STATUS"))
        .stdout(predicate::str::contains("Test wire"));
}

#[test]
fn test_ready_format_table() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);
    create_wire(&temp_dir, "Ready wire");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .arg("--format")
        .arg("table")
        .assert()
        .success()
        .stdout(predicate::str::contains("ID"))
        .stdout(predicate::str::contains("Ready wire"));
}

#[test]
fn test_show_format_table() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);
    let wire_id = create_wire(&temp_dir, "Show wire");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .arg("--format")
        .arg("table")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show wire"))
        .stdout(predicate::str::contains("TODO"));
}

#[test]
fn test_graph_format_table_not_supported() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    // Graph doesn't support table format - should error or fall back to json
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("graph")
        .arg("--format")
        .arg("table")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format"));
}

// When piped (which tests always are), default should be JSON
#[test]
fn test_list_default_is_json_when_piped() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);
    create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    // Should be valid JSON (not table) when piped
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
}
