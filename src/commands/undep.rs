use anyhow::Result;
use serde_json::json;
use wr::db;

pub fn run(wire_id: &str, depends_on: &str) -> Result<()> {
    let conn = db::open()?;

    db::remove_dependency(&conn, wire_id, depends_on)?;

    let output = json!({
        "wire_id": wire_id,
        "depends_on": depends_on,
        "action": "removed"
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
