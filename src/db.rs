use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

const WIRES_DIR: &str = ".wires";
const DB_NAME: &str = "wires.db";

/// Initialize a new wires database in the current directory
pub fn init(path: &Path) -> Result<()> {
    let wires_dir = path.join(WIRES_DIR);

    if wires_dir.exists() {
        return Err(anyhow!(
            "Wires already initialized at {}",
            wires_dir.display()
        ));
    }

    fs::create_dir(&wires_dir).context("Failed to create .wires directory")?;

    let db_path = wires_dir.join(DB_NAME);
    let conn = Connection::open(&db_path).context("Failed to create database")?;

    create_schema(&conn)?;

    Ok(())
}

/// Create the database schema
fn create_schema(conn: &Connection) -> Result<()> {
    // Enable WAL mode for concurrent access
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Create wires table
    conn.execute(
        "CREATE TABLE wires (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            priority INTEGER DEFAULT 0
        )",
        [],
    )?;

    // Create dependencies table
    conn.execute(
        "CREATE TABLE dependencies (
            wire_id TEXT NOT NULL,
            depends_on TEXT NOT NULL,
            FOREIGN KEY (wire_id) REFERENCES wires(id) ON DELETE CASCADE,
            FOREIGN KEY (depends_on) REFERENCES wires(id) ON DELETE CASCADE,
            PRIMARY KEY (wire_id, depends_on)
        )",
        [],
    )?;

    // Create indexes
    conn.execute("CREATE INDEX idx_status ON wires(status)", [])?;
    conn.execute("CREATE INDEX idx_priority ON wires(priority)", [])?;
    conn.execute("CREATE INDEX idx_deps_wire ON dependencies(wire_id)", [])?;
    conn.execute("CREATE INDEX idx_deps_on ON dependencies(depends_on)", [])?;

    Ok(())
}

/// Find the wires database by searching up the directory tree (git-style)
pub fn find_db() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    find_db_from(&current_dir)
}

/// Find the wires database starting from a specific directory
fn find_db_from(start: &Path) -> Result<PathBuf> {
    let mut current = start;

    loop {
        let wires_dir = current.join(WIRES_DIR);
        let db_path = wires_dir.join(DB_NAME);

        if db_path.exists() {
            return Ok(db_path);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => return Err(anyhow!("Not a wires repository")),
        }
    }
}

/// Open a connection to the wires database
pub fn open() -> Result<Connection> {
    let db_path = find_db()?;
    Connection::open(db_path).context("Failed to open database")
}

/// Insert a new wire into the database
pub fn insert_wire(conn: &Connection, wire: &crate::models::Wire) -> Result<()> {
    conn.execute(
        "INSERT INTO wires (id, title, description, status, created_at, updated_at, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            &wire.id,
            &wire.title,
            wire.description.as_deref().unwrap_or(""),
            wire.status.as_str(),
            &wire.created_at.to_string(),
            &wire.updated_at.to_string(),
            &wire.priority.to_string(),
        ],
    )?;
    Ok(())
}

/// List wires, optionally filtered by status
pub fn list_wires(
    conn: &Connection,
    status_filter: Option<&str>,
) -> Result<Vec<crate::models::Wire>> {
    use crate::models::{Status, Wire};
    use std::str::FromStr;

    let mut query = String::from(
        "SELECT id, title, description, status, created_at, updated_at, priority FROM wires",
    );

    if let Some(status) = status_filter {
        query.push_str(&format!(" WHERE status = '{}'", status));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut stmt = conn.prepare(&query)?;
    let wires = stmt.query_map([], |row| {
        let description: Option<String> = row.get(2)?;
        let description = description.filter(|s| !s.is_empty());

        Ok(Wire {
            id: row.get(0)?,
            title: row.get(1)?,
            description,
            status: Status::from_str(row.get::<_, String>(3)?.as_str())
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            priority: row.get(6)?,
        })
    })?;

    let mut result = Vec::new();
    for wire in wires {
        result.push(wire?);
    }

    Ok(result)
}

/// Get a wire with its dependency information
pub fn get_wire_with_deps(conn: &Connection, wire_id: &str) -> Result<crate::models::WireWithDeps> {
    use crate::models::{DependencyInfo, Status, Wire, WireWithDeps};
    use std::str::FromStr;

    // Get the wire
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, created_at, updated_at, priority
         FROM wires WHERE id = ?1",
    )?;

    let wire = stmt.query_row([wire_id], |row| {
        let description: Option<String> = row.get(2)?;
        let description = description.filter(|s| !s.is_empty());

        Ok(Wire {
            id: row.get(0)?,
            title: row.get(1)?,
            description,
            status: Status::from_str(row.get::<_, String>(3)?.as_str())
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            priority: row.get(6)?,
        })
    })?;

    // Get dependencies (wires this wire depends on)
    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.status
         FROM wires w
         JOIN dependencies d ON w.id = d.depends_on
         WHERE d.wire_id = ?1",
    )?;

    let depends_on = stmt
        .query_map([wire_id], |row| {
            Ok(DependencyInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                status: Status::from_str(row.get::<_, String>(2)?.as_str())
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Get blockers (wires that depend on this wire)
    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.status
         FROM wires w
         JOIN dependencies d ON w.id = d.wire_id
         WHERE d.depends_on = ?1",
    )?;

    let blocks = stmt
        .query_map([wire_id], |row| {
            Ok(DependencyInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                status: Status::from_str(row.get::<_, String>(2)?.as_str())
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WireWithDeps {
        wire,
        depends_on,
        blocks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_directory_and_database() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        init(path).unwrap();

        assert!(path.join(WIRES_DIR).exists());
        assert!(path.join(WIRES_DIR).join(DB_NAME).exists());
    }

    #[test]
    fn test_init_fails_if_already_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        init(path).unwrap();
        let result = init(path);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already initialized"));
    }

    #[test]
    fn test_schema_creation() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        init(path).unwrap();

        let db_path = path.join(WIRES_DIR).join(DB_NAME);
        let conn = Connection::open(db_path).unwrap();

        // Check that tables exist
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();

        assert!(tables.contains(&"wires".to_string()));
        assert!(tables.contains(&"dependencies".to_string()));

        // Check WAL mode
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_uppercase(), "WAL");
    }

    #[test]
    fn test_find_db_current_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        init(path).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();

        let result = find_db();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert!(result.unwrap().ends_with(DB_NAME));
    }

    #[test]
    fn test_find_db_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        init(path).unwrap();

        // Create subdirectory
        let sub_dir = path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let result = find_db_from(&sub_dir);

        assert!(result.is_ok());
        assert!(result.unwrap().ends_with(DB_NAME));
    }

    #[test]
    fn test_find_db_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = find_db_from(path);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a wires repository"));
    }
}
