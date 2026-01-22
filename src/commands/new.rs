use anyhow::Result;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use wr::db;
use wr::generate_id;
use wr::models::{Status, Wire};

pub fn run(title: &str, description: Option<&str>, priority: i32) -> Result<()> {
    let conn = db::open()?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let wire = Wire {
        id: generate_id(title),
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        status: Status::Todo,
        created_at: now,
        updated_at: now,
        priority,
    };

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
