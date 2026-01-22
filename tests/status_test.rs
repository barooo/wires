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
fn test_start_sets_in_progress() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("start")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "IN_PROGRESS");
}

#[test]
fn test_done_sets_done() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("done")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "DONE");
}

#[test]
fn test_cancel_sets_cancelled() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("cancel")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "CANCELLED");
}

#[test]
fn test_done_without_incomplete_deps_has_no_warnings() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("done")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("warnings").is_none() || json["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn test_start_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("start")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_done_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("done")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_cancel_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("cancel")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}
