//! Tag types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef};

/// A tag in Asana.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// The unique identifier for the tag.
    pub gid: Gid,
    /// The name of the tag.
    pub name: String,
    /// The color of the tag.
    pub color: Option<String>,
    /// Notes about the tag.
    pub notes: Option<String>,
    /// The workspace the tag belongs to.
    pub workspace: Option<ResourceRef>,
    /// When the tag was created.
    pub created_at: Option<String>,
    /// Permalink URL for the tag.
    pub permalink_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_tag() {
        let json = r#"{
            "gid": "123",
            "name": "Priority",
            "color": "red",
            "notes": "High priority items",
            "workspace": {"gid": "ws456", "name": "My Workspace"}
        }"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.gid, "123");
        assert_eq!(tag.name, "Priority");
        assert_eq!(tag.color, Some("red".to_string()));
    }

    #[test]
    fn test_deserialize_tag_minimal() {
        let json = r#"{
            "gid": "456",
            "name": "Bug"
        }"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.gid, "456");
        assert_eq!(tag.name, "Bug");
        assert!(tag.color.is_none());
    }
}
