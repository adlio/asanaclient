//! Section types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef};

/// A section in an Asana project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    /// The unique identifier for the section.
    pub gid: Gid,
    /// The name of the section.
    pub name: String,
    /// The project this section belongs to.
    pub project: Option<ResourceRef>,
    /// When the section was created.
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_section() {
        let json = r#"{
            "gid": "123",
            "name": "To Do",
            "project": {"gid": "proj456", "name": "My Project"},
            "created_at": "2024-01-15T10:00:00.000Z"
        }"#;
        let section: Section = serde_json::from_str(json).unwrap();
        assert_eq!(section.gid, "123");
        assert_eq!(section.name, "To Do");
        assert!(section.project.is_some());
    }

    #[test]
    fn test_deserialize_section_minimal() {
        let json = r#"{
            "gid": "456",
            "name": "Done"
        }"#;
        let section: Section = serde_json::from_str(json).unwrap();
        assert_eq!(section.gid, "456");
        assert_eq!(section.name, "Done");
        assert!(section.project.is_none());
    }
}
