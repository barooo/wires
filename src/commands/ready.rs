use anyhow::Result;
use wr::{
    db,
    format::{format_wire_table, print_json, Format},
};

pub fn run(format: Option<Format>) -> Result<()> {
    let format = Format::resolve(format);

    let conn = db::open()?;
    let wires = db::get_ready_wires(&conn)?;

    match format {
        Format::Json => print_json(&wires)?,
        Format::Table => print!("{}", format_wire_table(&wires)),
    }

    Ok(())
}
