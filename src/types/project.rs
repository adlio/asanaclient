//! Project types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, StatusColor, UserRef};
use super::custom_field::CustomFieldSetting;
use super::status_update::StatusUpdateRef;

/// An Asana project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// The unique identifier for the project.
    pub gid: Gid,
    /// The name of the project.
    pub name: String,
    /// The project's color (for display).
    pub color: Option<String>,
    /// Whether the project is archived.
    #[serde(default)]
    pub archived: bool,
    /// Whether the project is public to the organization.
    #[serde(default)]
    pub public: bool,
    /// The project owner.
    pub owner: Option<UserRef>,
    /// The team the project belongs to.
    pub team: Option<ResourceRef>,
    /// The workspace the project belongs to.
    pub workspace: Option<ResourceRef>,
    /// The current status update.
    pub current_status_update: Option<StatusUpdateRef>,
    /// The current status (deprecated, use current_status_update).
    pub current_status: Option<StatusUpdateRef>,
    /// Notes/description for the project.
    pub notes: Option<String>,
    /// HTML notes for the project.
    pub html_notes: Option<String>,
    /// When the project was created.
    pub created_at: Option<String>,
    /// When the project was last modified.
    pub modified_at: Option<String>,
    /// The due date of the project.
    pub due_date: Option<String>,
    /// The due date/time of the project.
    pub due_on: Option<String>,
    /// The start date of the project.
    pub start_on: Option<String>,
    /// The default view for the project (list, board, calendar, timeline).
    pub default_view: Option<String>,
    /// Whether the project is a template.
    #[serde(default)]
    pub is_template: bool,
    /// Permalink URL for the project.
    pub permalink_url: Option<String>,
    /// Icon for the project.
    pub icon: Option<String>,
    /// Custom field settings for this project.
    #[serde(default)]
    pub custom_field_settings: Vec<CustomFieldSetting>,
}

impl Project {
    /// Get the effective status color from either current_status_update or current_status.
    pub fn status_color(&self) -> StatusColor {
        self.current_status_update
            .as_ref()
            .or(self.current_status.as_ref())
            .and_then(|s| s.status_type)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_project() {
        let json = r#"{
            "gid": "123",
            "name": "My Project",
            "archived": false,
            "public": true,
            "notes": "Project description"
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.gid, "123");
        assert_eq!(project.name, "My Project");
        assert!(!project.archived);
        assert!(project.public);
    }

    #[test]
    fn test_project_status_color() {
        let json = r#"{
            "gid": "123",
            "name": "My Project",
            "current_status_update": {
                "gid": "456",
                "title": "On Track",
                "status_type": "green"
            }
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.status_color(), StatusColor::Green);
    }

    #[test]
    fn test_project_status_color_default() {
        let json = r#"{
            "gid": "123",
            "name": "My Project"
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.status_color(), StatusColor::None);
    }
}
