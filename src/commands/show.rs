use anyhow::{anyhow, Result};
use wr::{
    db,
    format::{format_wire_detail_table, print_json, Format},
};

pub fn run(wire_id: &str, format: Option<Format>) -> Result<()> {
    let format = Format::resolve(format);

    let conn = db::open()?;
    let wire_with_deps = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| anyhow!("Wire not found: {}", wire_id))?;

    match format {
        Format::Json => print_json(&wire_with_deps)?,
        Format::Table => print!("{}", format_wire_detail_table(&wire_with_deps)),
    }

    Ok(())
}
