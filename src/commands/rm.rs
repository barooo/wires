use anyhow::Result;
use serde_json::json;
use wr::db;
use wr::models::WireError;

pub fn run(id: &str) -> Result<()> {
    let conn = db::open()?;

    // Enable foreign keys for cascade delete to work
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Check if wire exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM wires WHERE id = ?1",
        [id],
        |row: &rusqlite::Row| row.get(0),
    )?;

    if exists == 0 {
        return Err(WireError::WireNotFound(id.to_string()).into());
    }

    // Delete the wire (dependencies are cascaded by foreign key)
    conn.execute("DELETE FROM wires WHERE id = ?1", [id])?;

    let output = json!({
        "id": id,
        "action": "deleted"
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
