use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Wire status values
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

/// A wire (task/item)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: Status,
    pub created_at: i64,
    pub updated_at: i64,
    pub priority: i32,
}

/// Wire with dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireWithDeps {
    #[serde(flatten)]
    pub wire: Wire,
    pub depends_on: Vec<DependencyInfo>,
    pub blocks: Vec<DependencyInfo>,
}

/// Dependency information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub id: String,
    pub title: String,
    pub status: Status,
}

/// A dependency relationship
#[derive(Debug, Clone)]
pub struct Dependency {
    pub wire_id: String,
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
