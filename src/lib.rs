//! # wires
//!
//! A lightweight local task tracker optimized for AI coding agents.
//!
//! `wires` provides a simple SQLite-backed task tracking system designed for
//! AI agents working on multi-step coding tasks. It supports dependency tracking
//! to help agents determine what to work on next.
//!
//! ## Features
//!
//! - **JSON output** for programmatic parsing (auto-detected when piped)
//! - **Human-readable tables** in terminal
//! - **Dependency tracking** with circular dependency prevention
//! - **Ready command** to find unblocked tasks
//! - **GraphViz DOT export** for visualization
//!
//! ## Modules
//!
//! - [`db`] - Database operations (init, open, CRUD, dependencies)
//! - [`models`] - Data structures (Wire, Status, WireWithDeps)
//! - [`mod@format`] - Output formatting (JSON, tables, TTY detection)
//!
//! ## Example
//!
//! ```no_run
//! use wr::db;
//!
//! // Open the database (searches up directory tree for .wires/)
//! let conn = db::open().expect("Failed to open database");
//!
//! // List ready wires
//! let ready = db::get_ready_wires(&conn).expect("Failed to get ready wires");
//! for wire in ready {
//!     println!("{}: {}", wire.id, wire.title);
//! }
//! ```

pub mod db;
pub mod format;
pub mod models;

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a unique 7-character hexadecimal ID from a title.
///
/// The ID is derived from a SHA-256 hash of the title combined with
/// the current timestamp in nanoseconds, ensuring uniqueness even
/// for identical titles.
///
/// # Arguments
///
/// * `title` - The wire title to generate an ID for
///
/// # Returns
///
/// A 7-character lowercase hexadecimal string
///
/// # Example
///
/// ```
/// let id = wr::generate_id("Implement feature X");
/// assert_eq!(id.len(), 7);
/// assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
pub fn generate_id(title: &str) -> String {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let input = format!("{}{}", title, timestamp_nanos);
    let hash = Sha256::digest(input.as_bytes());
    let hex = format!("{:x}", hash);

    hex[..7].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_length() {
        let id = generate_id("Test wire");
        assert_eq!(id.len(), 7);
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let id1 = generate_id("Test wire");
        let id2 = generate_id("Test wire");
        // Same title but different timestamps should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_hex_characters() {
        let id = generate_id("Test wire");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
