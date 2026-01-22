use anyhow::{anyhow, Result};
use serde_json::json;
use wr::db;

pub fn run(wire_id: &str) -> Result<()> {
    let conn = db::open()?;

    db::update_wire(&conn, wire_id, None, None, Some("IN_PROGRESS"), None)?;

    let wire = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| anyhow!("Wire not found: {}", wire_id))?;

    let output = json!({
        "id": wire.wire.id,
        "status": wire.wire.status,
        "updated_at": wire.wire.updated_at
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
