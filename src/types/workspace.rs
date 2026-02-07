//! Workspace types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::Gid;

/// An Asana workspace or organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    /// The unique identifier for the workspace.
    pub gid: Gid,
    /// The name of the workspace.
    pub name: String,
    /// Whether this is an organization (true) or a basic workspace (false).
    #[serde(default)]
    pub is_organization: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_workspace() {
        let json = r#"{"gid": "123", "name": "My Workspace", "is_organization": true}"#;
        let workspace: Workspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.gid, "123");
        assert_eq!(workspace.name, "My Workspace");
        assert!(workspace.is_organization);
    }

    #[test]
    fn test_deserialize_workspace_defaults() {
        let json = r#"{"gid": "456", "name": "Basic Workspace"}"#;
        let workspace: Workspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.gid, "456");
        assert!(!workspace.is_organization);
    }
}
