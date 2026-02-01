//! Common types shared across the Asana API.

use serde::{Deserialize, Serialize};

/// A globally unique identifier for an Asana resource.
pub type Gid = String;

/// A reference to a user with minimal information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRef {
    /// The unique identifier for the user.
    pub gid: Gid,
    /// The user's display name.
    pub name: Option<String>,
    /// The user's email address.
    pub email: Option<String>,
}

/// A reference to any Asana resource with minimal information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRef {
    /// The unique identifier for the resource.
    pub gid: Gid,
    /// The resource type (e.g., "project", "portfolio", "task").
    pub resource_type: Option<String>,
    /// The display name of the resource.
    pub name: Option<String>,
}

/// Color used for status updates and other UI elements.
///
/// Asana uses both color names (`green`, `yellow`, `red`, `blue`) and
/// semantic names (`on_track`, `at_risk`, `off_track`, `on_hold`, `complete`).
/// This enum accepts both forms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusColor {
    /// Green status - on track.
    #[serde(alias = "on_track")]
    Green,
    /// Yellow status - at risk.
    #[serde(alias = "at_risk")]
    Yellow,
    /// Red status - off track.
    #[serde(alias = "off_track")]
    Red,
    /// Blue status - on hold or complete.
    #[serde(alias = "on_hold", alias = "complete")]
    Blue,
    /// No status set.
    #[default]
    None,
}

/// Response wrapper for single-object API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct DataWrapper<T> {
    /// The wrapped data.
    pub data: T,
}

/// Response wrapper for paginated list API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ListWrapper<T> {
    /// The list of items.
    pub data: Vec<T>,
    /// Pagination information for fetching more results.
    pub next_page: Option<NextPage>,
}

/// Pagination cursor for fetching additional results.
#[derive(Debug, Clone, Deserialize)]
pub struct NextPage {
    /// The offset token for the next page.
    pub offset: String,
    /// The URI path for the next page (informational).
    pub path: Option<String>,
    /// The full URI for the next page (informational).
    pub uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color_default() {
        assert_eq!(StatusColor::default(), StatusColor::None);
    }

    #[test]
    fn test_deserialize_status_color() {
        // Color names
        let green: StatusColor = serde_json::from_str(r#""green""#).unwrap();
        assert_eq!(green, StatusColor::Green);

        let yellow: StatusColor = serde_json::from_str(r#""yellow""#).unwrap();
        assert_eq!(yellow, StatusColor::Yellow);

        // Semantic names (aliases)
        let on_track: StatusColor = serde_json::from_str(r#""on_track""#).unwrap();
        assert_eq!(on_track, StatusColor::Green);

        let at_risk: StatusColor = serde_json::from_str(r#""at_risk""#).unwrap();
        assert_eq!(at_risk, StatusColor::Yellow);

        let off_track: StatusColor = serde_json::from_str(r#""off_track""#).unwrap();
        assert_eq!(off_track, StatusColor::Red);

        let on_hold: StatusColor = serde_json::from_str(r#""on_hold""#).unwrap();
        assert_eq!(on_hold, StatusColor::Blue);

        let complete: StatusColor = serde_json::from_str(r#""complete""#).unwrap();
        assert_eq!(complete, StatusColor::Blue);
    }

    #[test]
    fn test_deserialize_user_ref() {
        let json = r#"{"gid": "123", "name": "Test User", "email": "test@example.com"}"#;
        let user: UserRef = serde_json::from_str(json).unwrap();
        assert_eq!(user.gid, "123");
        assert_eq!(user.name, Some("Test User".to_string()));
        assert_eq!(user.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_deserialize_data_wrapper() {
        let json = r#"{"data": {"gid": "123", "name": "Test"}}"#;
        let wrapper: DataWrapper<ResourceRef> = serde_json::from_str(json).unwrap();
        assert_eq!(wrapper.data.gid, "123");
    }
}
