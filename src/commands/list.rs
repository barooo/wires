use anyhow::Result;
use wr::db;

pub fn run(status_filter: Option<&str>) -> Result<()> {
    let conn = db::open()?;
    let wires = db::list_wires(&conn, status_filter)?;

    let output = serde_json::to_string(&wires)?;
    println!("{}", output);

    Ok(())
}
