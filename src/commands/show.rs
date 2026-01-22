use anyhow::{anyhow, Result};
use wr::db;

pub fn run(wire_id: &str) -> Result<()> {
    let conn = db::open()?;
    let wire_with_deps = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| anyhow!("Wire not found: {}", wire_id))?;

    let output = serde_json::to_string(&wire_with_deps)?;
    println!("{}", output);

    Ok(())
}
