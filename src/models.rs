//! Data models for wires.
//!
//! This module contains the core data structures used throughout the application:
//! - [`WireId`] - A validated 7-character hexadecimal wire identifier
//! - [`Status`] - Task status enum (TODO, IN_PROGRESS, DONE, CANCELLED)
//! - [`Wire`] - A task/item with title, description, status, and priority
//! - [`WireWithDeps`] - A wire with its dependency relationships
//! - [`DependencyInfo`] - Summary info about a dependent wire

use clap::ValueEnum;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A validated 7-character hexadecimal wire identifier.
///
/// Wire IDs are generated from a hash of the title and timestamp,
/// providing uniqueness while being short enough for human use.
///
/// # Validation
///
/// A valid WireId must be exactly 7 lowercase hexadecimal characters (0-9, a-f).
///
/// # Example
///
/// ```
/// use wr::models::WireId;
///
/// let id = WireId::new("a1b2c3d").unwrap();
/// assert_eq!(id.as_str(), "a1b2c3d");
///
/// // Invalid IDs are rejected
/// assert!(WireId::new("too_long_id").is_err());
/// assert!(WireId::new("abc").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WireId(String);

impl WireId {
    /// Creates a new WireId from a string, validating the format.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not exactly 7 lowercase hex characters.
    pub fn new(s: &str) -> Result<Self, WireIdError> {
        if s.len() != 7 {
            return Err(WireIdError::InvalidLength(s.len()));
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(WireIdError::InvalidCharacters);
        }
        Ok(WireId(s.to_lowercase()))
    }

    /// Creates a WireId without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure the string is a valid 7-character hex string.
    /// This is intended for use when reading from the database where data
    /// is known to be valid.
    pub(crate) fn from_trusted(s: String) -> Self {
        WireId(s)
    }

    /// Returns the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WireId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for WireId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for WireId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WireId::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Error type for invalid wire IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireIdError {
    /// ID is not exactly 7 characters
    InvalidLength(usize),
    /// ID contains non-hexadecimal characters
    InvalidCharacters,
}

impl fmt::Display for WireIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireIdError::InvalidLength(len) => {
                write!(f, "Wire ID must be 7 characters, got {}", len)
            }
            WireIdError::InvalidCharacters => {
                write!(f, "Wire ID must contain only hexadecimal characters")
            }
        }
    }
}

impl std::error::Error for WireIdError {}

impl FromSql for WireId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        // Trust database values are valid (we wrote them)
        Ok(WireId::from_trusted(s.to_string()))
    }
}

impl ToSql for WireId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(self.0.as_bytes())))
    }
}

/// Task status values.
///
/// Wires progress through these states:
/// - `Todo` - Not yet started
/// - `InProgress` - Currently being worked on
/// - `Done` - Completed successfully
/// - `Cancelled` - Abandoned or no longer needed
///
/// # Serialization
///
/// Statuses serialize as uppercase strings: `"TODO"`, `"IN_PROGRESS"`, `"DONE"`, `"CANCELLED"`.
///
/// # CLI Usage
///
/// Implements [`ValueEnum`] for use with clap. Accepts case-insensitive values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Status {
    #[serde(rename = "TODO")]
    #[value(alias = "TODO")]
    Todo,
    #[serde(rename = "IN_PROGRESS")]
    #[value(alias = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "DONE")]
    #[value(alias = "DONE")]
    Done,
    #[serde(rename = "CANCELLED")]
    #[value(alias = "CANCELLED")]
    Cancelled,
}

impl Status {
    /// Returns the string representation of the status.
    ///
    /// # Example
    ///
    /// ```
    /// use wr::models::Status;
    /// assert_eq!(Status::InProgress.as_str(), "IN_PROGRESS");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Status::Todo => "TODO",
            Status::InProgress => "IN_PROGRESS",
            Status::Done => "DONE",
            Status::Cancelled => "CANCELLED",
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TODO" => Ok(Status::Todo),
            "IN_PROGRESS" => Ok(Status::InProgress),
            "DONE" => Ok(Status::Done),
            "CANCELLED" => Ok(Status::Cancelled),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// A wire (task/item) in the tracker.
///
/// Wires are the fundamental unit of work tracking. Each wire has:
/// - A unique 7-character hex ID
/// - A title describing the task
/// - An optional detailed description
/// - A status indicating progress
/// - Timestamps for creation and last update
/// - A priority for ordering (higher = more important)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    /// Unique 7-character hexadecimal identifier
    pub id: WireId,
    /// Short description of the task
    pub title: String,
    /// Optional detailed description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Current status of the wire
    pub status: Status,
    /// Unix timestamp when the wire was created
    pub created_at: i64,
    /// Unix timestamp when the wire was last updated
    pub updated_at: i64,
    /// Priority level (higher values = higher priority)
    pub priority: i32,
}

/// A wire with its full dependency information.
///
/// This struct includes the wire itself plus lists of:
/// - Wires this wire depends on (must complete before this one)
/// - Wires that depend on this wire (blocked until this completes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireWithDeps {
    /// The wire itself (fields are flattened in JSON)
    #[serde(flatten)]
    pub wire: Wire,
    /// Wires this wire depends on
    pub depends_on: Vec<DependencyInfo>,
    /// Wires that are blocked by this wire
    pub blocks: Vec<DependencyInfo>,
}

/// Summary information about a wire in a dependency relationship.
///
/// Used to display dependency information without loading full wire details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Wire ID
    pub id: WireId,
    /// Wire title
    pub title: String,
    /// Current status
    pub status: Status,
}

/// A dependency relationship between two wires.
///
/// Represents that `wire_id` depends on `depends_on`, meaning
/// `depends_on` must be completed before `wire_id` is ready to work on.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The wire that has the dependency
    pub wire_id: WireId,
    /// The wire it depends on
    pub depends_on: WireId,
}

/// Domain-specific errors for wire operations.
///
/// These errors represent business logic failures that can be pattern-matched
/// for specific handling, unlike generic string errors.
#[derive(Debug, Clone)]
pub enum WireError {
    /// The `.wires` directory was not found in any parent directory
    NotARepository,
    /// A wires repository already exists at the specified location
    AlreadyInitialized(String),
    /// The specified wire ID does not exist
    WireNotFound(String),
    /// Adding this dependency would create a circular dependency chain
    CircularDependency(Vec<String>),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireError::NotARepository => write!(f, "Not a wires repository"),
            WireError::AlreadyInitialized(path) => {
                write!(f, "Wires already initialized at {}", path)
            }
            WireError::WireNotFound(id) => write!(f, "Wire not found: {}", id),
            WireError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for WireError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_id_valid() {
        let id = WireId::new("a1b2c3d").unwrap();
        assert_eq!(id.as_str(), "a1b2c3d");
    }

    #[test]
    fn test_wire_id_wrong_length() {
        assert!(WireId::new("abc").is_err());
        assert!(WireId::new("a1b2c3d4").is_err());
    }

    #[test]
    fn test_wire_id_non_hex() {
        assert!(WireId::new("a1b2c3g").is_err()); // 'g' is not hex
    }

    #[test]
    fn test_wire_id_serialization() {
        let id = WireId::new("a1b2c3d").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""a1b2c3d""#);

        let parsed: WireId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(Status::Todo.as_str(), "TODO");
        assert_eq!(Status::InProgress.as_str(), "IN_PROGRESS");
        assert_eq!(Status::Done.as_str(), "DONE");
        assert_eq!(Status::Cancelled.as_str(), "CANCELLED");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!("TODO".parse::<Status>().unwrap(), Status::Todo);
        assert_eq!("IN_PROGRESS".parse::<Status>().unwrap(), Status::InProgress);
        assert_eq!("DONE".parse::<Status>().unwrap(), Status::Done);
        assert_eq!("CANCELLED".parse::<Status>().unwrap(), Status::Cancelled);
        assert!("INVALID".parse::<Status>().is_err());
    }

    #[test]
    fn test_status_serde() {
        let status = Status::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""IN_PROGRESS""#);

        let parsed: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Status::InProgress);
    }

    #[test]
    fn test_wire_serialization() {
        let wire = Wire {
            id: WireId::new("a3f2c1b").unwrap(),
            title: "Test wire".to_string(),
            description: Some("Test description".to_string()),
            status: Status::Todo,
            created_at: 1704067200,
            updated_at: 1704067200,
            priority: 0,
        };

        let json = serde_json::to_string(&wire).unwrap();
        assert!(json.contains(r#""id":"a3f2c1b""#));
        assert!(json.contains(r#""status":"TODO""#));
    }

    #[test]
    fn test_wire_serialization_without_description() {
        let wire = Wire {
            id: WireId::new("a3f2c1b").unwrap(),
            title: "Test wire".to_string(),
            description: None,
            status: Status::Todo,
            created_at: 1704067200,
            updated_at: 1704067200,
            priority: 0,
        };

        let json = serde_json::to_string(&wire).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_wire_error_display() {
        assert_eq!(
            WireError::NotARepository.to_string(),
            "Not a wires repository"
        );
        assert_eq!(
            WireError::AlreadyInitialized("/path/.wires".to_string()).to_string(),
            "Wires already initialized at /path/.wires"
        );
        assert_eq!(
            WireError::WireNotFound("abc1234".to_string()).to_string(),
            "Wire not found: abc1234"
        );
        assert_eq!(
            WireError::CircularDependency(vec!["a".to_string(), "b".to_string(), "a".to_string()])
                .to_string(),
            "Circular dependency detected: a -> b -> a"
        );
    }
}
