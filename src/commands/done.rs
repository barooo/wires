use anyhow::Result;
use serde_json::json;
use wr::db;
use wr::models::{Status, WireError};

pub fn run(wire_id: &str) -> Result<()> {
    let conn = db::open()?;

    // Check for incomplete dependencies
    let incomplete_deps = db::check_incomplete_dependencies(&conn, wire_id)?;

    // Update status to DONE
    db::update_wire(&conn, wire_id, None, None, Some(Status::Done), None)?;

    let wire = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| WireError::WireNotFound(wire_id.to_string()))?;

    let mut output = json!({
        "id": wire.wire.id,
        "status": wire.wire.status,
        "updated_at": wire.wire.updated_at
    });

    // Add warnings if there are incomplete dependencies
    if !incomplete_deps.is_empty() {
        let warnings: Vec<_> = incomplete_deps
            .iter()
            .map(|dep| {
                json!({
                    "type": "incomplete_dependency",
                    "wire_id": dep.id,
                    "status": dep.status
                })
            })
            .collect();

        output["warnings"] = json!(warnings);
    }

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
