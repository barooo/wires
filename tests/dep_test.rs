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
fn test_dep_adds_dependency() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_b)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "added");
    assert_eq!(json["wire_id"], wire_a);
    assert_eq!(json["depends_on"], wire_b);

    // Verify dependency was added by checking show output
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_a)
        .output()
        .unwrap();

    let show_json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let deps = show_json["depends_on"].as_array().unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0]["id"], wire_b);
}

#[test]
fn test_dep_detects_direct_cycle() {
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

    // Try to make B depend on A (would create cycle)
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_b)
        .arg(&wire_a)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Circular dependency detected"));
}

#[test]
fn test_dep_detects_indirect_cycle() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");
    let wire_c = create_wire(&temp_dir, "Wire C");

    // Create chain: A -> B -> C
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_b)
        .assert()
        .success();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_b)
        .arg(&wire_c)
        .assert()
        .success();

    // Try to make C depend on A (would create cycle: A -> B -> C -> A)
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_c)
        .arg(&wire_a)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Circular dependency detected"));
}

#[test]
fn test_dep_self_dependency() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");

    // Try to make wire depend on itself
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_a)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Circular dependency detected"));
}

#[test]
fn test_dep_nonexistent_wire() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Wire not found"));
}

#[test]
fn test_undep_removes_dependency() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    // Add dependency
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_a)
        .arg(&wire_b)
        .assert()
        .success();

    // Remove dependency
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("undep")
        .arg(&wire_a)
        .arg(&wire_b)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "removed");

    // Verify dependency was removed
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_a)
        .output()
        .unwrap();

    let show_json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let deps = show_json["depends_on"].as_array().unwrap();
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_undep_nonexistent_dependency() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    // Remove non-existent dependency (should succeed silently)
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("undep")
        .arg(&wire_a)
        .arg(&wire_b)
        .assert()
        .success();
}

#[test]
fn test_dep_allows_diamond_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");
    let wire_c = create_wire(&temp_dir, "Wire C");
    let wire_d = create_wire(&temp_dir, "Wire D");

    // Create diamond: D -> B, D -> C, B -> A, C -> A
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_d)
        .arg(&wire_b)
        .assert()
        .success();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_d)
        .arg(&wire_c)
        .assert()
        .success();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_b)
        .arg(&wire_a)
        .assert()
        .success();

    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("dep")
        .arg(&wire_c)
        .arg(&wire_a)
        .assert()
        .success();

    // Verify all dependencies
    let show_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("show")
        .arg(&wire_d)
        .output()
        .unwrap();

    let show_json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let deps = show_json["depends_on"].as_array().unwrap();
    assert_eq!(deps.len(), 2);
}
