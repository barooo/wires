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

#[test]
fn test_update_title() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Original title");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(&wire_id)
        .arg("--title")
        .arg("New title")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify the title was updated
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    assert_eq!(json["title"], "New title");
}

#[test]
fn test_update_status() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(&wire_id)
        .arg("--status")
        .arg("IN_PROGRESS")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "IN_PROGRESS");
}

#[test]
fn test_update_priority() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(&wire_id)
        .arg("--priority")
        .arg("5")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["priority"], 5);
}

#[test]
fn test_update_multiple_fields() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(&wire_id)
        .arg("--status")
        .arg("IN_PROGRESS")
        .arg("--priority")
        .arg("2")
        .assert()
        .success();

    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    assert_eq!(json["status"], "IN_PROGRESS");
    assert_eq!(json["priority"], 2);
}

#[test]
fn test_update_invalid_status() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg(&wire_id)
        .arg("--status")
        .arg("INVALID")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid status"));
}

#[test]
fn test_update_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("update")
        .arg("nonexistent")
        .arg("--status")
        .arg("DONE")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}
