use anyhow::Result;
use serde_json::json;
use wr::db;
use wr::models::Wire;

pub fn run(title: &str, description: Option<&str>, priority: i32) -> Result<()> {
    let conn = db::open()?;

    let wire = Wire::new(title, description, priority)?;

    db::insert_wire(&conn, &wire)?;

    let output = json!({
        "id": wire.id,
        "title": wire.title,
        "status": wire.status,
        "priority": wire.priority,
        "created_at": wire.created_at
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
