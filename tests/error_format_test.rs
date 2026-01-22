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

// When piped (which tests always are), errors should be JSON
#[test]
fn test_error_is_json_when_piped() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg("nonexistent")
        .output()
        .unwrap();

    assert!(!output.status.success());

    // stderr should be valid JSON
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    assert!(json.get("error").is_some());
}

#[test]
fn test_not_initialized_error_is_json_when_piped() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    assert!(json["error"].as_str().unwrap().contains("wires repository"));
}

#[test]
fn test_invalid_status_error_is_json_when_piped() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    // Create a wire first
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Test")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wire_id = json["id"].as_str().unwrap();

    // Try invalid status
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(wire_id)
        .arg("--status")
        .arg("INVALID")
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    assert!(json["error"].as_str().unwrap().contains("Invalid status"));
}
