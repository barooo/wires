use assert_cmd::Command;
use std::thread;
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
fn test_concurrent_wire_creation() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    let dir_path = temp_dir.path().to_path_buf();

    // Spawn multiple threads that each create a wire
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let dir = dir_path.clone();
            thread::spawn(move || {
                let output = Command::cargo_bin("wr")
                    .unwrap()
                    .current_dir(&dir)
                    .arg("new")
                    .arg(format!("Concurrent wire {}", i))
                    .output()
                    .unwrap();

                assert!(
                    output.status.success(),
                    "Thread {} failed: {}",
                    i,
                    String::from_utf8_lossy(&output.stderr)
                );

                let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                json["id"].as_str().unwrap().to_string()
            })
        })
        .collect();

    // Wait for all threads and collect IDs
    let ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all 5 wires were created with unique IDs
    assert_eq!(ids.len(), 5);

    // Check all IDs are unique
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), 5, "All IDs should be unique");

    // Verify we can list all 5 wires
    let list_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    let wires: Vec<serde_json::Value> = serde_json::from_slice(&list_output.stdout).unwrap();
    assert_eq!(wires.len(), 5);
}

#[test]
fn test_concurrent_read_and_write() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(&temp_dir);

    // Create initial wire
    let output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("new")
        .arg("Initial wire")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let wire_id = json["id"].as_str().unwrap().to_string();

    let dir_path = temp_dir.path().to_path_buf();
    let wire_id_clone = wire_id.clone();

    // Spawn threads: some reading, some writing
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let dir = dir_path.clone();
            let id = wire_id_clone.clone();
            thread::spawn(move || {
                if i % 2 == 0 {
                    // Read operation
                    let output = Command::cargo_bin("wr")
                        .unwrap()
                        .current_dir(&dir)
                        .arg("show")
                        .arg(&id)
                        .output()
                        .unwrap();

                    assert!(
                        output.status.success(),
                        "Read thread {} failed: {}",
                        i,
                        String::from_utf8_lossy(&output.stderr)
                    );
                } else {
                    // Write operation - create new wire
                    let output = Command::cargo_bin("wr")
                        .unwrap()
                        .current_dir(&dir)
                        .arg("new")
                        .arg(format!("Wire from thread {}", i))
                        .output()
                        .unwrap();

                    assert!(
                        output.status.success(),
                        "Write thread {} failed: {}",
                        i,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            })
        })
        .collect();

    // Wait for all threads
    for h in handles {
        h.join().unwrap();
    }

    // Verify database is consistent - should have 6 wires (1 initial + 5 from odd threads)
    let list_output = Command::cargo_bin("wr")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("list")
        .output()
        .unwrap();

    let wires: Vec<serde_json::Value> = serde_json::from_slice(&list_output.stdout).unwrap();
    assert_eq!(wires.len(), 6);
}
