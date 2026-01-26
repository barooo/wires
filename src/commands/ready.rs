use anyhow::Result;
use wr::{
    db,
    format::{format_wire_table, print_json, Format},
    models::WireWithDeps,
};

pub fn run(format: Option<Format>) -> Result<()> {
    let format = Format::resolve(format);

    let conn = db::open()?;
    let wires = db::get_ready_wires(&conn)?;

    match format {
        Format::Json => print_json(&wires)?,
        Format::Table => {
            // Ready wires have no incomplete dependencies by definition
            let wires_with_deps: Vec<WireWithDeps> =
                wires.into_iter().map(WireWithDeps::from).collect();
            print!("{}", format_wire_table(&wires_with_deps))
        }
    }

    Ok(())
}
