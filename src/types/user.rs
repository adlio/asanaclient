//! User types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::Gid;

/// An Asana user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// The unique identifier for the user.
    pub gid: Gid,
    /// The user's display name.
    pub name: String,
    /// The user's email address.
    pub email: Option<String>,
    /// URL to the user's profile photo.
    pub photo: Option<UserPhoto>,
}

/// Photo information for a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPhoto {
    /// URL to the 21x21 pixel version of the photo.
    pub image_21x21: Option<String>,
    /// URL to the 27x27 pixel version of the photo.
    pub image_27x27: Option<String>,
    /// URL to the 36x36 pixel version of the photo.
    pub image_36x36: Option<String>,
    /// URL to the 60x60 pixel version of the photo.
    pub image_60x60: Option<String>,
    /// URL to the 128x128 pixel version of the photo.
    pub image_128x128: Option<String>,
}

/// A favorite item from a user's favorites list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteItem {
    /// The unique identifier for the resource.
    pub gid: Gid,
    /// The resource type (e.g., "project", "portfolio").
    pub resource_type: String,
    /// The display name of the resource.
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_user() {
        let json = r#"{
            "gid": "123",
            "name": "Test User",
            "email": "test@example.com"
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.gid, "123");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_deserialize_favorite_item() {
        let json = r#"{
            "gid": "456",
            "resource_type": "project",
            "name": "My Project"
        }"#;
        let item: FavoriteItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.gid, "456");
        assert_eq!(item.resource_type, "project");
        assert_eq!(item.name, Some("My Project".to_string()));
    }
}
