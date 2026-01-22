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
