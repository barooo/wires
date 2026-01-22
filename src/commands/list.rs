use anyhow::{anyhow, Result};
use wr::{
    db,
    format::{format_wire_table, print_json, Format},
};

pub fn run(status_filter: Option<&str>, format_str: Option<&str>) -> Result<()> {
    let format = Format::from_str_or_auto(format_str).map_err(|e| anyhow!(e))?;

    let conn = db::open()?;
    let wires = db::list_wires(&conn, status_filter)?;

    match format {
        Format::Json => print_json(&wires)?,
        Format::Table => print!("{}", format_wire_table(&wires)),
    }

    Ok(())
}
