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

#[test]
fn test_graph_empty() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("graph")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["nodes"].as_array().unwrap().is_empty());
    assert!(json["edges"].as_array().unwrap().is_empty());
}

#[test]
fn test_graph_nodes_only() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("graph")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let nodes = json["nodes"].as_array().unwrap();
    let edges = json["edges"].as_array().unwrap();

    assert_eq!(nodes.len(), 2);
    assert!(edges.is_empty());

    // Verify nodes contain expected fields
    let node_ids: Vec<&str> = nodes.iter().map(|n| n["id"].as_str().unwrap()).collect();
    assert!(node_ids.contains(&wire_a.as_str()));
    assert!(node_ids.contains(&wire_b.as_str()));

    // Verify node structure
    let node = &nodes[0];
    assert!(node.get("id").is_some());
    assert!(node.get("title").is_some());
    assert!(node.get("status").is_some());
    assert!(node.get("priority").is_some());
}

#[test]
fn test_graph_with_edges() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let wire_a = create_wire(&temp_dir, "Wire A");
    let wire_b = create_wire(&temp_dir, "Wire B");
    let wire_c = create_wire(&temp_dir, "Wire C");

    // A depends on B, B depends on C
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

    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("graph")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let nodes = json["nodes"].as_array().unwrap();
    let edges = json["edges"].as_array().unwrap();

    assert_eq!(nodes.len(), 3);
    assert_eq!(edges.len(), 2);

    // Verify edges represent dependencies (from depends on to)
    let edge_pairs: Vec<(String, String)> = edges
        .iter()
        .map(|e| {
            (
                e["from"].as_str().unwrap().to_string(),
                e["to"].as_str().unwrap().to_string(),
            )
        })
        .collect();

    assert!(edge_pairs.contains(&(wire_a.clone(), wire_b.clone())));
    assert!(edge_pairs.contains(&(wire_b.clone(), wire_c.clone())));
}

#[test]
fn test_graph_format_json_explicit() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    create_wire(&temp_dir, "Wire A");

    // Explicit --format json should work
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("graph")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("nodes").is_some());
    assert!(json.get("edges").is_some());
}
