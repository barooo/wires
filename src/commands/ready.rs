use anyhow::Result;
use wr::db;

pub fn run() -> Result<()> {
    let conn = db::open()?;
    let wires = db::get_ready_wires(&conn)?;

    let output = serde_json::to_string(&wires)?;
    println!("{}", output);

    Ok(())
}
