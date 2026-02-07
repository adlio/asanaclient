//! Project template types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, UserRef};

/// A project template in Asana.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectTemplate {
    /// The unique identifier for the template.
    pub gid: Gid,
    /// The name of the template.
    pub name: String,
    /// Description of the template.
    pub description: Option<String>,
    /// HTML description of the template.
    pub html_description: Option<String>,
    /// The owner of the template.
    pub owner: Option<UserRef>,
    /// The team the template belongs to.
    pub team: Option<ResourceRef>,
    /// Whether the template is public.
    #[serde(default)]
    pub public: bool,
    /// Requested dates for template instantiation.
    #[serde(default)]
    pub requested_dates: Vec<RequestedDate>,
    /// Requested roles for template instantiation.
    #[serde(default)]
    pub requested_roles: Vec<RequestedRole>,
    /// The color of projects created from this template.
    pub color: Option<String>,
}

/// A date variable requested during template instantiation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedDate {
    /// The GID of the date variable.
    pub gid: Gid,
    /// The name/label of the date variable.
    pub name: Option<String>,
    /// Description of what this date is for.
    pub description: Option<String>,
}

/// A role requested during template instantiation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedRole {
    /// The GID of the role.
    pub gid: Gid,
    /// The name/label of the role.
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_project_template() {
        let json = r#"{
            "gid": "123",
            "name": "Sprint Template",
            "description": "Template for sprint planning",
            "public": true,
            "requested_dates": [
                {"gid": "date1", "name": "Sprint Start", "description": "When the sprint begins"}
            ],
            "requested_roles": [
                {"gid": "role1", "name": "Sprint Lead"}
            ]
        }"#;
        let template: ProjectTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(template.gid, "123");
        assert_eq!(template.name, "Sprint Template");
        assert!(template.public);
        assert_eq!(template.requested_dates.len(), 1);
        assert_eq!(template.requested_roles.len(), 1);
    }

    #[test]
    fn test_deserialize_project_template_minimal() {
        let json = r#"{
            "gid": "456",
            "name": "Basic Template"
        }"#;
        let template: ProjectTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(template.gid, "456");
        assert_eq!(template.name, "Basic Template");
        assert!(template.requested_dates.is_empty());
        assert!(template.requested_roles.is_empty());
    }

    #[test]
    fn test_deserialize_requested_date() {
        let json = r#"{
            "gid": "date123",
            "name": "Project Deadline",
            "description": "When the project must be completed"
        }"#;
        let date: RequestedDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.gid, "date123");
        assert_eq!(date.name, Some("Project Deadline".to_string()));
    }
}
