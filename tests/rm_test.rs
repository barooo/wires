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
fn test_rm_deletes_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Wire to delete");

    // Delete the wire
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("rm")
        .arg(&wire_id)
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["id"], wire_id);
    assert_eq!(json["action"], "deleted");

    // Verify wire no longer exists
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_rm_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("rm")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_rm_cascades_dependencies_where_wire_depends_on_others() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    // A depends on B
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_b)
        .assert()
        .success();

    // Delete A - dependency record should be removed
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("rm")
        .arg(&wire_a)
        .assert()
        .success();

    // B should still exist and have no blockers
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_b)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let blocks = json["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 0);
}

#[test]
fn test_rm_cascades_dependencies_where_others_depend_on_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    // A depends on B
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_b)
        .assert()
        .success();

    // Delete B - dependency record should be removed
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("rm")
        .arg(&wire_b)
        .assert()
        .success();

    // A should still exist and have no dependencies
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_a)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let deps = json["depends_on"].as_array().unwrap();
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_rm_not_initialized() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("rm")
        .arg("someid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a wires repository"));
}
