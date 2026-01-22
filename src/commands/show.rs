use anyhow::Result;
use wr::{
    db,
    format::{format_wire_detail_table, print_json, Format},
    models::WireError,
};

pub fn run(wire_id: &str, format: Option<Format>) -> Result<()> {
    let format = Format::resolve(format);

    let conn = db::open()?;
    let wire_with_deps = db::get_wire_with_deps(&conn, wire_id)
        .map_err(|_| WireError::WireNotFound(wire_id.to_string()))?;

    match format {
        Format::Json => print_json(&wire_with_deps)?,
        Format::Table => print!("{}", format_wire_detail_table(&wire_with_deps)),
    }

    Ok(())
}
