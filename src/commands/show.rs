use anyhow::{anyhow, Result};
use wr::{
    db,
    format::{format_wire_detail_table, print_json, Format},
};

pub fn run(wire_id: &str, format_str: Option<&str>) -> Result<()> {
    let format = Format::from_str_or_auto(format_str).map_err(|e| anyhow!(e))?;

    let conn = db::open()?;
    let wire_with_deps = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| anyhow!("Wire not found: {}", wire_id))?;

    match format {
        Format::Json => print_json(&wire_with_deps)?,
        Format::Table => print!("{}", format_wire_detail_table(&wire_with_deps)),
    }

    Ok(())
}
