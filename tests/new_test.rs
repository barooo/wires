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

#[test]
fn test_new_creates_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let mut cmd = Command::cargo_bin("wr").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("Test wire")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test wire"))
        .stdout(predicate::str::contains("TODO"));
}

#[test]
fn test_new_with_description() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let mut cmd = Command::cargo_bin("wr").unwrap();
    let output = cmd
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Test wire")
        .arg("--description")
        .arg("Test description")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "Test wire");
    assert_eq!(json["status"], "TODO");
}

#[test]
fn test_new_with_priority() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let mut cmd = Command::cargo_bin("wr").unwrap();
    let output = cmd
        .current_dir(&temp_dir)
        .arg("new")
        .arg("High priority wire")
        .arg("--priority")
        .arg("1")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["priority"], 1);
}

#[test]
fn test_new_generates_unique_ids() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let output1 = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Wire 1")
        .output()
        .unwrap();

    let output2 = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Wire 1")
        .output()
        .unwrap();

    let json1: serde_json::Value = serde_json::from_slice(&output1.stdout).unwrap();
    let json2: serde_json::Value = serde_json::from_slice(&output2.stdout).unwrap();

    assert_ne!(json1["id"], json2["id"]);
}

#[test]
fn test_new_fails_without_init() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Test wire")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a wires repository"));
}
