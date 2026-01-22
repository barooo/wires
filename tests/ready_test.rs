use assert_cmd::Command;
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

fn create_wire_with_priority(dir: &TempDir, title: &str, priority: i32) -> String {
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("new")
        .arg(title)
        .arg(format!("--priority={}", priority))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wr new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["id"].as_str().unwrap().to_string()
}

fn add_dependency(dir: &TempDir, wire_id: &str, depends_on: &str) {
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("dep")
        .arg(wire_id)
        .arg(depends_on)
        .assert()
        .success();
}

fn start_wire(dir: &TempDir, wire_id: &str) {
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("start")
        .arg(wire_id)
        .assert()
        .success();
}

fn done_wire(dir: &TempDir, wire_id: &str) {
    Command::cargo_bin("wr")
        .unwrap()
        .current_dir(dir)
        .arg("done")
        .arg(wire_id)
        .assert()
        .success();
}

#[test]
fn test_ready_empty() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[test]
fn test_ready_shows_todo_wires() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    create_wire(&temp_dir, "Wire 1");
    create_wire(&temp_dir, "Wire 2");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();
    assert_eq!(wires.len(), 2);
}

#[test]
fn test_ready_prioritizes_in_progress_over_todo() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_todo = create_wire(&temp_dir, "TODO wire");
    let wire_in_progress = create_wire(&temp_dir, "In progress wire");

    start_wire(&temp_dir, &wire_in_progress);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    assert_eq!(wires.len(), 2);
    // IN_PROGRESS should be first
    assert_eq!(wires[0]["status"], "IN_PROGRESS");
    assert_eq!(wires[0]["id"], wire_in_progress);
    assert_eq!(wires[1]["status"], "TODO");
    assert_eq!(wires[1]["id"], wire_todo);
}

#[test]
fn test_ready_sorts_by_priority_within_status() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_low = create_wire_with_priority(&temp_dir, "Low priority", -1);
    let wire_high = create_wire_with_priority(&temp_dir, "High priority", 5);
    let wire_medium = create_wire_with_priority(&temp_dir, "Medium priority", 2);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    assert_eq!(wires.len(), 3);
    // Should be sorted by priority descending
    assert_eq!(wires[0]["id"], wire_high);
    assert_eq!(wires[1]["id"], wire_medium);
    assert_eq!(wires[2]["id"], wire_low);
}

#[test]
fn test_ready_excludes_done_wires() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_todo = create_wire(&temp_dir, "TODO wire");
    let wire_done = create_wire(&temp_dir, "Done wire");

    done_wire(&temp_dir, &wire_done);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0]["id"], wire_todo);
}

#[test]
fn test_ready_excludes_wires_with_incomplete_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let dep_wire = create_wire(&temp_dir, "Dependency");
    let blocked_wire = create_wire(&temp_dir, "Blocked wire");
    let ready_wire = create_wire(&temp_dir, "Ready wire");

    add_dependency(&temp_dir, &blocked_wire, &dep_wire);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    // Should only show ready_wire and dep_wire (no dependencies)
    // blocked_wire has incomplete dependency
    assert_eq!(wires.len(), 2);

    let ids: Vec<&str> = wires.iter().map(|w| w["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&ready_wire.as_str()));
    assert!(ids.contains(&dep_wire.as_str()));
    assert!(!ids.contains(&blocked_wire.as_str()));
}

#[test]
fn test_ready_includes_wires_with_completed_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let dep_wire = create_wire(&temp_dir, "Dependency");
    let wire_with_dep = create_wire(&temp_dir, "Wire with dependency");

    add_dependency(&temp_dir, &wire_with_dep, &dep_wire);
    done_wire(&temp_dir, &dep_wire);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    // Should include wire_with_dep since its dependency is DONE
    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0]["id"], wire_with_dep);
}

#[test]
fn test_ready_complex_scenario() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    // Create wires with different priorities and statuses
    let in_progress_high = create_wire_with_priority(&temp_dir, "In progress high", 10);
    let in_progress_low = create_wire_with_priority(&temp_dir, "In progress low", 1);
    let todo_high = create_wire_with_priority(&temp_dir, "TODO high", 8);
    let todo_low = create_wire_with_priority(&temp_dir, "TODO low", 2);
    let blocked = create_wire(&temp_dir, "Blocked");
    let blocker = create_wire(&temp_dir, "Blocker");

    start_wire(&temp_dir, &in_progress_high);
    start_wire(&temp_dir, &in_progress_low);
    add_dependency(&temp_dir, &blocked, &blocker);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("ready")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wires = json.as_array().unwrap();

    // Should have: 2 IN_PROGRESS, 2 TODO, 1 blocker (no deps)
    // blocked should not appear
    assert_eq!(wires.len(), 5);

    // Order should be:
    // 1. IN_PROGRESS high priority (10)
    // 2. IN_PROGRESS low priority (1)
    // 3. TODO high priority (8)
    // 4. TODO low priority (2)
    // 5. Blocker (0 priority, TODO)
    assert_eq!(wires[0]["id"], in_progress_high);
    assert_eq!(wires[1]["id"], in_progress_low);
    assert_eq!(wires[2]["id"], todo_high);
    assert_eq!(wires[3]["id"], todo_low);
    assert_eq!(wires[4]["id"], blocker);
}
