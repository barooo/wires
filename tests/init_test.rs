use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_creates_database() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("wr").unwrap();

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"))
        .stdout(predicate::str::contains(".wires/wires.db"));

    // Verify database was created
    assert!(temp_dir.path().join(".wires").join("wires.db").exists());
}

#[test]
fn test_init_fails_if_already_initialized() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize once
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success();

    // Try to initialize again
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_init_output_is_json() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("wr").unwrap();

    let output = cmd.current_dir(&temp_dir).arg("init").output().unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "initialized");
    assert!(json["path"].as_str().unwrap().ends_with(".wires/wires.db"));
}
