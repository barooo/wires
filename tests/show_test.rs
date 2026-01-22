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
fn test_show_displays_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["id"], wire_id);
    assert_eq!(json["title"], "Test wire");
    assert_eq!(json["status"], "TODO");
}

#[test]
fn test_show_includes_empty_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["depends_on"].is_array());
    assert_eq!(json["depends_on"].as_array().unwrap().len(), 0);
    assert!(json["blocks"].is_array());
    assert_eq!(json["blocks"].as_array().unwrap().len(), 0);
}

#[test]
fn test_show_fails_for_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_show_output_is_valid_json() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_id = create_wire(&temp_dir, "Test wire");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_id)
        .output()
        .unwrap();

    // Should parse without error
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Check all required fields exist
    assert!(json.get("id").is_some());
    assert!(json.get("title").is_some());
    assert!(json.get("status").is_some());
    assert!(json.get("created_at").is_some());
    assert!(json.get("updated_at").is_some());
    assert!(json.get("priority").is_some());
    assert!(json.get("depends_on").is_some());
    assert!(json.get("blocks").is_some());
}
