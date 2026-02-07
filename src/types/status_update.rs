//! Status update types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, StatusColor, UserRef};

/// A status update on a project or portfolio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusUpdate {
    /// The unique identifier for the status update.
    pub gid: Gid,
    /// The title of the status update.
    pub title: Option<String>,
    /// The text content of the status update.
    pub text: Option<String>,
    /// The HTML content of the status update.
    pub html_text: Option<String>,
    /// The color indicating the status (green, yellow, red, blue).
    pub status_type: Option<StatusColor>,
    /// When the status update was created.
    pub created_at: Option<String>,
    /// The user who created the status update.
    pub created_by: Option<UserRef>,
    /// The parent resource (project or portfolio).
    pub parent: Option<ResourceRef>,
}

/// A compact reference to a status update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusUpdateRef {
    /// The unique identifier for the status update.
    pub gid: Gid,
    /// The title of the status update.
    pub title: Option<String>,
    /// The text content of the status update.
    pub text: Option<String>,
    /// The color indicating the status.
    pub status_type: Option<StatusColor>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_status_update() {
        let json = r#"{
            "gid": "123",
            "title": "On Track",
            "text": "Everything is going well",
            "status_type": "green",
            "created_at": "2024-01-15T10:30:00.000Z"
        }"#;
        let status: StatusUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(status.gid, "123");
        assert_eq!(status.title, Some("On Track".to_string()));
        assert_eq!(status.status_type, Some(StatusColor::Green));
    }

    #[test]
    fn test_deserialize_status_update_ref() {
        let json = r#"{
            "gid": "456",
            "title": "At Risk",
            "status_type": "yellow"
        }"#;
        let status_ref: StatusUpdateRef = serde_json::from_str(json).unwrap();
        assert_eq!(status_ref.gid, "456");
        assert_eq!(status_ref.status_type, Some(StatusColor::Yellow));
    }
}
