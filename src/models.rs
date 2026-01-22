//! Data models for wires.
//!
//! This module contains the core data structures used throughout the application:
//! - [`Status`] - Task status enum (TODO, IN_PROGRESS, DONE, CANCELLED)
//! - [`Wire`] - A task/item with title, description, status, and priority
//! - [`WireWithDeps`] - A wire with its dependency relationships
//! - [`DependencyInfo`] - Summary info about a dependent wire

use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "TODO")]
    Todo,
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "DONE")]
    Done,
    #[serde(rename = "CANCELLED")]
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
    pub id: String,
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
    pub id: String,
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
    pub wire_id: String,
    /// The wire it depends on
    pub depends_on: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            id: "a3f2c1b".to_string(),
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
            id: "a3f2c1b".to_string(),
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
}
