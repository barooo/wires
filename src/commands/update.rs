use anyhow::{anyhow, Result};
use serde_json::json;
use wr::db;

pub fn run(
    wire_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<&str>,
    priority: Option<i32>,
) -> Result<()> {
    let conn = db::open()?;

    // Validate status if provided
    if let Some(s) = status {
        use std::str::FromStr;
        use wr::models::Status;
        Status::from_str(s).map_err(|e| anyhow!("{}", e))?;
    }

    // Perform update
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
        .map_err(|_| anyhow!("Wire not found: {}", wire_id))?;

    let output = json!({
        "id": wire.wire.id,
        "status": wire.wire.status,
        "priority": wire.wire.priority,
        "updated_at": wire.wire.updated_at
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
