//! Database operations for wires.
//!
//! This module handles all SQLite database operations including:
//! - Initialization and schema creation
//! - Wire CRUD operations
//! - Dependency management with circular dependency detection
//! - Finding ready-to-work wires
//!
//! The database is stored in `.wires/wires.db` and uses WAL mode for
//! concurrent access support.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::WireError;

const WIRES_DIR: &str = ".wires";
const DB_NAME: &str = "wires.db";

/// Initializes a new wires database in the specified directory.
///
/// Creates a `.wires/` directory containing a SQLite database with
/// the required schema (wires and dependencies tables).
///
/// # Arguments
///
/// * `path` - The directory where `.wires/` should be created
///
/// # Errors
///
/// Returns an error if:
/// - The `.wires/` directory already exists
/// - Directory creation fails
/// - Database creation fails
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use wr::db;
///
/// db::init(Path::new("/path/to/project")).expect("Failed to initialize");
/// ```
pub fn init(path: &Path) -> Result<()> {
    let wires_dir = path.join(WIRES_DIR);

    if wires_dir.exists() {
        return Err(WireError::AlreadyInitialized(wires_dir.display().to_string()).into());
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

/// Finds the wires database by searching up the directory tree.
///
/// Like git, this searches from the current directory upward until it
/// finds a `.wires/` directory containing the database.
///
/// # Errors
///
/// Returns an error if no `.wires/` directory is found in any parent directory.
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
            None => return Err(WireError::NotARepository.into()),
        }
    }
}

/// Opens a connection to the wires database.
///
/// Searches for the database using [`find_db`], then opens a connection to it.
///
/// # Errors
///
/// Returns an error if no database is found or the connection fails.
///
/// # Example
///
/// ```no_run
/// use wr::db;
///
/// let conn = db::open().expect("Not in a wires repository");
/// ```
pub fn open() -> Result<Connection> {
    let db_path = find_db()?;
    Connection::open(db_path).context("Failed to open database")
}

/// Inserts a new wire into the database.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire` - The wire to insert
///
/// # Errors
///
/// Returns an error if the insert fails (e.g., duplicate ID).
pub fn insert_wire(conn: &Connection, wire: &crate::models::Wire) -> Result<()> {
    conn.execute(
        "INSERT INTO wires (id, title, description, status, created_at, updated_at, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            &wire.id,
            &wire.title,
            wire.description.as_deref().unwrap_or(""),
            wire.status.as_str(),
            wire.created_at,
            wire.updated_at,
            wire.priority,
        ],
    )?;
    Ok(())
}

/// Updates one or more fields of a wire.
///
/// Only fields with `Some` values are updated. The `updated_at` timestamp
/// is automatically set to the current time.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire_id` - ID of the wire to update
/// * `title` - New title, if changing
/// * `description` - New description (`Some(Some("desc"))` to set, `Some(None)` to clear)
/// * `status` - New status
/// * `priority` - New priority value
pub fn update_wire(
    conn: &Connection,
    wire_id: &str,
    title: Option<&str>,
    description: Option<Option<&str>>,
    status: Option<crate::models::Status>,
    priority: Option<i32>,
) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let mut query_parts = Vec::new();

    if title.is_some() {
        query_parts.push("title = ?");
    }

    if description.is_some() {
        query_parts.push("description = ?");
    }

    if status.is_some() {
        query_parts.push("status = ?");
    }

    if priority.is_some() {
        query_parts.push("priority = ?");
    }

    if query_parts.is_empty() {
        return Ok(());
    }

    query_parts.push("updated_at = ?");

    let query = format!("UPDATE wires SET {} WHERE id = ?", query_parts.join(", "));

    // Build params dynamically
    let mut stmt = conn.prepare(&query)?;
    let mut param_index = 1;

    if let Some(t) = title {
        stmt.raw_bind_parameter(param_index, t)?;
        param_index += 1;
    }

    if let Some(d) = description {
        stmt.raw_bind_parameter(param_index, d.unwrap_or(""))?;
        param_index += 1;
    }

    if let Some(ref s) = status {
        stmt.raw_bind_parameter(param_index, s.as_str())?;
        param_index += 1;
    }

    if let Some(p) = priority {
        stmt.raw_bind_parameter(param_index, p)?;
        param_index += 1;
    }

    stmt.raw_bind_parameter(param_index, now)?;
    param_index += 1;

    stmt.raw_bind_parameter(param_index, wire_id)?;

    stmt.raw_execute()?;

    Ok(())
}

/// Checks for incomplete dependencies of a wire.
///
/// Returns a list of wires that this wire depends on which are not yet `DONE`.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire_id` - ID of the wire to check
///
/// # Returns
///
/// A vector of [`DependencyInfo`](crate::models::DependencyInfo) for each incomplete dependency.
pub fn check_incomplete_dependencies(
    conn: &Connection,
    wire_id: &str,
) -> Result<Vec<crate::models::DependencyInfo>> {
    use crate::models::{DependencyInfo, Status};
    use std::str::FromStr;

    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.status
         FROM wires w
         JOIN dependencies d ON w.id = d.depends_on
         WHERE d.wire_id = ?1 AND w.status != 'DONE'",
    )?;

    let deps = stmt
        .query_map([wire_id], |row| {
            Ok(DependencyInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                status: Status::from_str(row.get::<_, String>(2)?.as_str())
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(deps)
}

/// Map a row to a Wire struct (shared by list_wires, get_wire_with_deps, get_ready_wires)
fn wire_from_row(row: &rusqlite::Row) -> rusqlite::Result<crate::models::Wire> {
    use crate::models::{Status, Wire};
    use std::str::FromStr;

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
}

/// Map a row to a DependencyInfo struct
fn dependency_info_from_row(
    row: &rusqlite::Row,
) -> rusqlite::Result<crate::models::DependencyInfo> {
    use crate::models::{DependencyInfo, Status};
    use std::str::FromStr;

    Ok(DependencyInfo {
        id: row.get(0)?,
        title: row.get(1)?,
        status: Status::from_str(row.get::<_, String>(2)?.as_str())
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
    })
}

/// Fetch dependency relationships for a wire.
///
/// Returns (depends_on, blocks) where:
/// - depends_on: wires this wire depends on
/// - blocks: wires that depend on this wire
fn fetch_wire_deps(
    conn: &Connection,
    wire_id: &str,
) -> Result<(
    Vec<crate::models::DependencyInfo>,
    Vec<crate::models::DependencyInfo>,
)> {
    // Get dependencies (wires this wire depends on)
    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.status
         FROM wires w
         JOIN dependencies d ON w.id = d.depends_on
         WHERE d.wire_id = ?1",
    )?;

    let depends_on = stmt
        .query_map([wire_id], dependency_info_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    // Get blockers (wires that depend on this wire)
    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.status
         FROM wires w
         JOIN dependencies d ON w.id = d.wire_id
         WHERE d.depends_on = ?1",
    )?;

    let blocks = stmt
        .query_map([wire_id], dependency_info_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((depends_on, blocks))
}

/// Lists wires, optionally filtered by status.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `status_filter` - Optional status to filter by
///
/// # Returns
///
/// A vector of wires ordered by creation date (newest first).
pub fn list_wires(
    conn: &Connection,
    status_filter: Option<crate::models::Status>,
) -> Result<Vec<crate::models::Wire>> {
    if let Some(status) = status_filter {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, created_at, updated_at, priority
             FROM wires WHERE status = ? ORDER BY created_at DESC",
        )?;
        let wires = stmt
            .query_map([status.as_str()], wire_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(wires)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, created_at, updated_at, priority
             FROM wires ORDER BY created_at DESC",
        )?;
        let wires = stmt
            .query_map([], wire_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(wires)
    }
}

/// Lists wires with their dependency information, optionally filtered by status.
///
/// Similar to `list_wires` but returns full `WireWithDeps` objects including
/// dependency relationships.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `status_filter` - Optional status to filter by
///
/// # Returns
///
/// A vector of wires with dependencies, ordered by creation date (newest first).
pub fn list_wires_with_deps(
    conn: &Connection,
    status_filter: Option<crate::models::Status>,
) -> Result<Vec<crate::models::WireWithDeps>> {
    use crate::models::WireWithDeps;

    let wires = list_wires(conn, status_filter)?;

    wires
        .into_iter()
        .map(|wire| {
            let (depends_on, blocks) = fetch_wire_deps(conn, wire.id.as_str())?;
            Ok(WireWithDeps {
                wire,
                depends_on,
                blocks,
            })
        })
        .collect()
}

/// Gets a wire with its full dependency information.
///
/// Returns the wire along with lists of wires it depends on and wires that depend on it.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire_id` - ID of the wire to fetch
///
/// # Errors
///
/// Returns an error if the wire is not found.
pub fn get_wire_with_deps(conn: &Connection, wire_id: &str) -> Result<crate::models::WireWithDeps> {
    use crate::models::WireWithDeps;

    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, created_at, updated_at, priority
         FROM wires WHERE id = ?1",
    )?;

    let wire = stmt.query_row([wire_id], wire_from_row)?;
    let (depends_on, blocks) = fetch_wire_deps(conn, wire_id)?;

    Ok(WireWithDeps {
        wire,
        depends_on,
        blocks,
    })
}

/// Check if adding a dependency would create a cycle using DFS
fn would_create_cycle(
    conn: &Connection,
    wire_id: &str,
    depends_on: &str,
) -> Result<Option<Vec<String>>> {
    use std::collections::{HashSet, VecDeque};

    // If wire depends on itself, that's a cycle
    if wire_id == depends_on {
        return Ok(Some(vec![wire_id.to_string(), wire_id.to_string()]));
    }

    // DFS to check if we can reach wire_id starting from depends_on
    let mut visited = HashSet::new();
    let mut stack = VecDeque::new();
    let mut parent_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    stack.push_back(depends_on.to_string());

    while let Some(current) = stack.pop_back() {
        if visited.contains(&current) {
            continue;
        }

        visited.insert(current.clone());

        // If we reached the original wire, we found a cycle
        if current == wire_id {
            // Reconstruct the cycle path
            let mut path = vec![wire_id.to_string()];
            let mut node = depends_on.to_string();

            while node != wire_id {
                path.push(node.clone());
                if let Some(parent) = parent_map.get(&node) {
                    node = parent.clone();
                } else {
                    break;
                }
            }

            path.push(wire_id.to_string());
            path.reverse();
            return Ok(Some(path));
        }

        // Get all wires that current depends on
        let mut stmt = conn.prepare("SELECT depends_on FROM dependencies WHERE wire_id = ?1")?;

        let deps: Vec<String> = stmt
            .query_map([&current], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        for dep in deps {
            if !visited.contains(&dep) {
                parent_map.insert(dep.clone(), current.clone());
                stack.push_back(dep);
            }
        }
    }

    Ok(None)
}

/// Adds a dependency between two wires.
///
/// Creates a dependency where `wire_id` depends on `depends_on`, meaning
/// `depends_on` must be completed before `wire_id` is ready to work on.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire_id` - The wire that has the dependency
/// * `depends_on` - The wire it depends on
///
/// # Errors
///
/// Returns an error if:
/// - Either wire does not exist
/// - The dependency would create a circular dependency
pub fn add_dependency(conn: &Connection, wire_id: &str, depends_on: &str) -> Result<()> {
    // Check if both wires exist
    let wire_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM wires WHERE id = ?1",
        [wire_id],
        |row| row.get(0),
    )?;

    if wire_exists == 0 {
        return Err(WireError::WireNotFound(wire_id.to_string()).into());
    }

    let depends_on_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM wires WHERE id = ?1",
        [depends_on],
        |row| row.get(0),
    )?;

    if depends_on_exists == 0 {
        return Err(WireError::WireNotFound(depends_on.to_string()).into());
    }

    // Check for circular dependency
    if let Some(cycle) = would_create_cycle(conn, wire_id, depends_on)? {
        return Err(WireError::CircularDependency(cycle).into());
    }

    // Add the dependency
    conn.execute(
        "INSERT OR IGNORE INTO dependencies (wire_id, depends_on) VALUES (?1, ?2)",
        [wire_id, depends_on],
    )?;

    Ok(())
}

/// Removes a dependency between two wires.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `wire_id` - The wire that has the dependency
/// * `depends_on` - The wire it depends on
pub fn remove_dependency(conn: &Connection, wire_id: &str, depends_on: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM dependencies WHERE wire_id = ?1 AND depends_on = ?2",
        [wire_id, depends_on],
    )?;

    Ok(())
}

/// Gets wires that are ready to work on.
///
/// A wire is ready if:
/// - Its status is `TODO` or `IN_PROGRESS`
/// - All wires it depends on have status `DONE`
///
/// Results are sorted by:
/// 1. Status (`IN_PROGRESS` first, then `TODO`)
/// 2. Priority (higher priority first)
///
/// This is the primary function for AI agents to determine what to work on next.
///
/// # Example
///
/// ```no_run
/// use wr::db;
///
/// let conn = db::open().expect("Failed to open database");
/// let ready = db::get_ready_wires(&conn).expect("Failed to get ready wires");
///
/// if let Some(next) = ready.first() {
///     println!("Next task: {} - {}", next.id, next.title);
/// }
/// ```
pub fn get_ready_wires(conn: &Connection) -> Result<Vec<crate::models::Wire>> {
    let query = "
        SELECT w.id, w.title, w.description, w.status, w.created_at, w.updated_at, w.priority
        FROM wires w
        WHERE w.status IN ('TODO', 'IN_PROGRESS')
        AND NOT EXISTS (
            SELECT 1 FROM dependencies d
            JOIN wires dep ON d.depends_on = dep.id
            WHERE d.wire_id = w.id
            AND dep.status != 'DONE'
        )
        ORDER BY
            CASE w.status
                WHEN 'IN_PROGRESS' THEN 0
                WHEN 'TODO' THEN 1
            END,
            w.priority DESC
    ";

    let mut stmt = conn.prepare(query)?;
    let wires = stmt
        .query_map([], wire_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(wires)
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

    // Helper to set up a test database with schema
    fn setup_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        init(path).unwrap();
        let db_path = path.join(WIRES_DIR).join(DB_NAME);
        let conn = Connection::open(db_path).unwrap();
        (temp_dir, conn)
    }

    // Helper to insert a test wire
    fn insert_test_wire(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO wires (id, title, status, created_at, updated_at, priority)
             VALUES (?1, ?2, 'TODO', 0, 0, 0)",
            [id, &format!("Wire {}", id)],
        )
        .unwrap();
    }

    // Helper to insert a dependency
    fn insert_test_dep(conn: &Connection, wire_id: &str, depends_on: &str) {
        conn.execute(
            "INSERT INTO dependencies (wire_id, depends_on) VALUES (?1, ?2)",
            [wire_id, depends_on],
        )
        .unwrap();
    }

    #[test]
    fn test_cycle_detection_self_reference() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a");

        let result = would_create_cycle(&conn, "a", "a").unwrap();

        assert!(result.is_some());
        let cycle = result.unwrap();
        assert_eq!(cycle, vec!["a", "a"]);
    }

    #[test]
    fn test_cycle_detection_direct_cycle() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a");
        insert_test_wire(&conn, "b");

        // a depends on b
        insert_test_dep(&conn, "a", "b");

        // Would b -> a create a cycle? Yes: b -> a -> b
        let result = would_create_cycle(&conn, "b", "a").unwrap();

        assert!(result.is_some());
    }

    #[test]
    fn test_cycle_detection_indirect_cycle() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a");
        insert_test_wire(&conn, "b");
        insert_test_wire(&conn, "c");

        // Chain: a -> b -> c
        insert_test_dep(&conn, "a", "b");
        insert_test_dep(&conn, "b", "c");

        // Would c -> a create a cycle? Yes: c -> a -> b -> c
        let result = would_create_cycle(&conn, "c", "a").unwrap();

        assert!(result.is_some());
    }

    #[test]
    fn test_cycle_detection_no_cycle() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a");
        insert_test_wire(&conn, "b");
        insert_test_wire(&conn, "c");

        // a -> b (no chain to c)
        insert_test_dep(&conn, "a", "b");

        // Would c -> a create a cycle? No, there's no path from a to c
        let result = would_create_cycle(&conn, "c", "a").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_cycle_detection_diamond_allowed() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a");
        insert_test_wire(&conn, "b");
        insert_test_wire(&conn, "c");
        insert_test_wire(&conn, "d");

        // Diamond: d -> b, d -> c, b -> a, c -> a
        insert_test_dep(&conn, "d", "b");
        insert_test_dep(&conn, "d", "c");
        insert_test_dep(&conn, "b", "a");
        insert_test_dep(&conn, "c", "a");

        // This is a valid DAG (diamond shape), not a cycle
        // Adding another dep from d to a should be allowed
        let result = would_create_cycle(&conn, "d", "a").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_fetch_wire_deps_empty() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");

        let (depends_on, blocks) = fetch_wire_deps(&conn, "a1b2c3d").unwrap();

        assert!(depends_on.is_empty());
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_fetch_wire_deps_with_dependencies() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");
        insert_test_wire(&conn, "b2c3d4e");
        insert_test_dep(&conn, "a1b2c3d", "b2c3d4e"); // a depends on b

        let (depends_on, blocks) = fetch_wire_deps(&conn, "a1b2c3d").unwrap();

        assert_eq!(depends_on.len(), 1);
        assert_eq!(depends_on[0].id.as_str(), "b2c3d4e");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_fetch_wire_deps_with_blocks() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");
        insert_test_wire(&conn, "b2c3d4e");
        insert_test_dep(&conn, "b2c3d4e", "a1b2c3d"); // b depends on a, so a blocks b

        let (depends_on, blocks) = fetch_wire_deps(&conn, "a1b2c3d").unwrap();

        assert!(depends_on.is_empty());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id.as_str(), "b2c3d4e");
    }

    #[test]
    fn test_fetch_wire_deps_both_directions() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");
        insert_test_wire(&conn, "b2c3d4e");
        insert_test_wire(&conn, "c3d4e5f");

        // a depends on b, c depends on a
        insert_test_dep(&conn, "a1b2c3d", "b2c3d4e");
        insert_test_dep(&conn, "c3d4e5f", "a1b2c3d");

        let (depends_on, blocks) = fetch_wire_deps(&conn, "a1b2c3d").unwrap();

        assert_eq!(depends_on.len(), 1);
        assert_eq!(depends_on[0].id.as_str(), "b2c3d4e");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id.as_str(), "c3d4e5f");
    }

    #[test]
    fn test_list_wires_with_deps_empty() {
        let (_temp_dir, conn) = setup_test_db();

        let result = list_wires_with_deps(&conn, None).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_list_wires_with_deps_includes_dependencies() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");
        insert_test_wire(&conn, "b2c3d4e");
        insert_test_dep(&conn, "a1b2c3d", "b2c3d4e");

        let result = list_wires_with_deps(&conn, None).unwrap();

        assert_eq!(result.len(), 2);

        // Find the wire that has a dependency
        let wire_a = result
            .iter()
            .find(|w| w.wire.id.as_str() == "a1b2c3d")
            .unwrap();
        assert_eq!(wire_a.depends_on.len(), 1);
        assert_eq!(wire_a.depends_on[0].id.as_str(), "b2c3d4e");

        // Find the wire that is depended on
        let wire_b = result
            .iter()
            .find(|w| w.wire.id.as_str() == "b2c3d4e")
            .unwrap();
        assert_eq!(wire_b.blocks.len(), 1);
        assert_eq!(wire_b.blocks[0].id.as_str(), "a1b2c3d");
    }

    #[test]
    fn test_list_wires_with_deps_respects_status_filter() {
        let (_temp_dir, conn) = setup_test_db();
        insert_test_wire(&conn, "a1b2c3d");

        // Change wire to DONE
        conn.execute("UPDATE wires SET status = 'DONE' WHERE id = 'a1b2c3d'", [])
            .unwrap();

        // Filter by TODO should return empty
        let todo_result = list_wires_with_deps(&conn, Some(crate::models::Status::Todo)).unwrap();
        assert!(todo_result.is_empty());

        // Filter by DONE should return the wire
        let done_result = list_wires_with_deps(&conn, Some(crate::models::Status::Done)).unwrap();
        assert_eq!(done_result.len(), 1);
    }
}
