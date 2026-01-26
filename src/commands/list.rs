use anyhow::Result;
use wr::{
    db,
    format::{format_wire_table, print_json, Format},
    models::Status,
};

pub fn run(status_filter: Option<Status>, format: Option<Format>) -> Result<()> {
    let format = Format::resolve(format);

    let conn = db::open()?;
    let wires_with_deps = db::list_wires_with_deps(&conn, status_filter)?;

    match format {
        Format::Json => {
            // For JSON, extract just the wires to maintain backward compatibility
            let wires: Vec<_> = wires_with_deps.iter().map(|wd| &wd.wire).collect();
            print_json(&wires)?
        }
        Format::Table => print!("{}", format_wire_table(&wires_with_deps)),
    }

    Ok(())
}
