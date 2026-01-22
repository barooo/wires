use anyhow::Result;
use serde_json::json;
use std::env;
use wr::db;

pub fn run() -> Result<()> {
    let current_dir = env::current_dir()?;
    db::init(&current_dir)?;

    let wires_path = current_dir.join(".wires").join("wires.db");
    let output = json!({
        "status": "initialized",
        "path": wires_path.display().to_string()
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
