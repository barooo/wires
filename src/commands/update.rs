use anyhow::Result;
use serde_json::json;
use wr::db;
use wr::models::{Status, WireError};

pub fn run(
    wire_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<Status>,
    priority: Option<i32>,
) -> Result<()> {
    let conn = db::open()?;

    db::update_wire(
        &conn,
        wire_id,
        title,
        description.map(Some),
        status,
        priority,
    )?;

    // Fetch updated wire
    let wire = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| WireError::WireNotFound(wire_id.to_string()))?;

    let output = json!({
        "id": wire.wire.id,
        "status": wire.wire.status,
        "priority": wire.wire.priority,
        "updated_at": wire.wire.updated_at
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
