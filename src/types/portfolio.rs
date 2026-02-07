//! Portfolio types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, StatusColor, UserRef};
use super::project::Project;
use super::status_update::StatusUpdateRef;

/// An Asana portfolio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    /// The unique identifier for the portfolio.
    pub gid: Gid,
    /// The name of the portfolio.
    pub name: String,
    /// The color of the portfolio.
    pub color: Option<String>,
    /// The portfolio owner.
    pub owner: Option<UserRef>,
    /// The workspace the portfolio belongs to.
    pub workspace: Option<ResourceRef>,
    /// The current status update.
    pub current_status_update: Option<StatusUpdateRef>,
    /// When the portfolio was created.
    pub created_at: Option<String>,
    /// The user who created the portfolio.
    pub created_by: Option<UserRef>,
    /// Permalink URL for the portfolio.
    pub permalink_url: Option<String>,
    /// Whether the portfolio is public.
    #[serde(default)]
    pub public: bool,
}

impl Portfolio {
    /// Get the effective status color from current_status_update.
    pub fn status_color(&self) -> StatusColor {
        self.current_status_update
            .as_ref()
            .and_then(|s| s.status_type)
            .unwrap_or_default()
    }
}

/// An item in a portfolio (either a project or nested portfolio).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "resource_type", rename_all = "snake_case")]
pub enum PortfolioItem {
    /// A project in the portfolio.
    Project(Box<Project>),
    /// A nested portfolio.
    Portfolio(Box<Portfolio>),
}

impl PortfolioItem {
    /// Get the GID of the item.
    pub fn gid(&self) -> &str {
        match self {
            Self::Project(p) => &p.gid,
            Self::Portfolio(p) => &p.gid,
        }
    }

    /// Get the name of the item.
    pub fn name(&self) -> &str {
        match self {
            Self::Project(p) => &p.name,
            Self::Portfolio(p) => &p.name,
        }
    }

    /// Get the status color of the item.
    pub fn status_color(&self) -> StatusColor {
        match self {
            Self::Project(p) => p.status_color(),
            Self::Portfolio(p) => p.status_color(),
        }
    }
}

/// A compact reference to a portfolio item for initial listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioItemRef {
    /// The unique identifier.
    pub gid: Gid,
    /// The resource type ("project" or "portfolio").
    pub resource_type: String,
    /// The display name.
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_portfolio() {
        let json = r#"{
            "gid": "123",
            "name": "My Portfolio",
            "color": "light-blue",
            "public": true
        }"#;
        let portfolio: Portfolio = serde_json::from_str(json).unwrap();
        assert_eq!(portfolio.gid, "123");
        assert_eq!(portfolio.name, "My Portfolio");
        assert!(portfolio.public);
    }

    #[test]
    fn test_deserialize_portfolio_item_project() {
        let json = r#"{
            "resource_type": "project",
            "gid": "456",
            "name": "Project A"
        }"#;
        let item: PortfolioItem = serde_json::from_str(json).unwrap();
        assert!(matches!(item, PortfolioItem::Project(_)));
        assert_eq!(item.gid(), "456");
        assert_eq!(item.name(), "Project A");
    }

    #[test]
    fn test_deserialize_portfolio_item_portfolio() {
        let json = r#"{
            "resource_type": "portfolio",
            "gid": "789",
            "name": "Nested Portfolio"
        }"#;
        let item: PortfolioItem = serde_json::from_str(json).unwrap();
        assert!(matches!(item, PortfolioItem::Portfolio(_)));
        assert_eq!(item.gid(), "789");
    }

    #[test]
    fn test_portfolio_status_color() {
        let json = r#"{
            "gid": "123",
            "name": "My Portfolio",
            "current_status_update": {
                "gid": "456",
                "status_type": "yellow"
            }
        }"#;
        let portfolio: Portfolio = serde_json::from_str(json).unwrap();
        assert_eq!(portfolio.status_color(), StatusColor::Yellow);
    }
}
