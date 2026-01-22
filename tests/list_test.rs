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

fn create_wire(dir: &TempDir, title: &str) {
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("new")
        .arg(title)
        .assert()
        .success();
}

#[test]
fn test_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[test]
fn test_list_shows_wires() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    create_wire(&temp_dir, "Wire 1");
    create_wire(&temp_dir, "Wire 2");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();
    assert_eq!(wires.len(), 2);
}

#[test]
fn test_list_filter_by_status() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    create_wire(&temp_dir, "Wire 1");
    create_wire(&temp_dir, "Wire 2");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .arg("--status")
        .arg("TODO")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();
    assert_eq!(wires.len(), 2);

    // All should have TODO status
    for wire in wires {
        assert_eq!(wire["status"], "TODO");
    }
}

#[test]
fn test_list_filter_returns_empty_for_nonmatching_status() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    create_wire(&temp_dir, "Wire 1");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .arg("--status")
        .arg("DONE")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 0);
}
