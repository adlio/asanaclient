//! Portfolio API endpoints.

use crate::types::requests::{
    AddItemData, AddItemRequest, AddMembersData, AddMembersRequest, CreatePortfolioData,
    CreatePortfolioRequest, RemoveItemData, RemoveItemRequest, RemoveMembersData,
    RemoveMembersRequest, UpdatePortfolioData, UpdatePortfolioRequest,
};
use crate::types::{Portfolio, PortfolioItemRef, Project, StatusUpdate};
use crate::{Client, Error};

/// Fields to request for a basic portfolio fetch.
pub const PORTFOLIO_FIELDS: &str = "gid,name,color,owner,owner.name,workspace,\
    current_status_update,current_status_update.gid,current_status_update.status_type,\
    current_status_update.title,current_status_update.text,\
    created_at,created_by,permalink_url,public";

/// Fields to request for portfolio items.
pub const PORTFOLIO_ITEMS_FIELDS: &str = "gid,resource_type,name";

/// Maximum recursion depth for fetching nested portfolios.
pub const MAX_PORTFOLIO_DEPTH: usize = 5;

/// API for portfolio operations.
pub struct PortfoliosApi<'a> {
    client: &'a Client,
}

impl<'a> PortfoliosApi<'a> {
    /// Create a new portfolios API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a portfolio by its GID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let portfolio = client.portfolios().get("12345").await?;
    /// println!("Portfolio: {}", portfolio.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, gid: &str) -> Result<Portfolio, Error> {
        let path = format!("/portfolios/{}", gid);
        let query = [("opt_fields", PORTFOLIO_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Get the items (projects and nested portfolios) in a portfolio.
    ///
    /// Returns compact references to the items. Use `get_items_full` for
    /// detailed information about each item.
    pub async fn items(&self, gid: &str) -> Result<Vec<PortfolioItemRef>, Error> {
        let path = format!("/portfolios/{}/items", gid);
        let query = [("opt_fields", PORTFOLIO_ITEMS_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    /// Get status updates for a portfolio.
    pub async fn status_updates(&self, gid: &str) -> Result<Vec<StatusUpdate>, Error> {
        let path = format!("/portfolios/{}/status_updates", gid);
        let query = [(
            "opt_fields",
            "gid,title,text,html_text,status_type,created_at,created_by,created_by.name",
        )];
        self.client.get_all(&path, &query).await
    }

    /// Get the latest status update for a portfolio.
    pub async fn latest_status_update(&self, gid: &str) -> Result<Option<StatusUpdate>, Error> {
        let updates = self.status_updates(gid).await?;
        Ok(updates.into_iter().next())
    }

    // ========== Write Operations ==========

    /// Create a new portfolio.
    pub async fn create(&self, data: CreatePortfolioData) -> Result<Portfolio, Error> {
        let path = "/portfolios".to_string();
        let request = CreatePortfolioRequest { data };
        self.client.post(&path, &request).await
    }

    /// Update a portfolio.
    pub async fn update(&self, gid: &str, data: UpdatePortfolioData) -> Result<Portfolio, Error> {
        let path = format!("/portfolios/{}", gid);
        let request = UpdatePortfolioRequest { data };
        self.client.put(&path, &request).await
    }

    /// Delete a portfolio.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/portfolios/{}", gid);
        self.client.delete(&path).await
    }

    /// Add an item (project) to a portfolio.
    pub async fn add_item(&self, portfolio_gid: &str, item_gid: &str) -> Result<(), Error> {
        let path = format!("/portfolios/{}/addItem", portfolio_gid);
        let request = AddItemRequest {
            data: AddItemData {
                item: item_gid.to_string(),
                insert_before: None,
                insert_after: None,
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add an item with positioning.
    pub async fn add_item_with_position(
        &self,
        portfolio_gid: &str,
        item_gid: &str,
        insert_before: Option<&str>,
        insert_after: Option<&str>,
    ) -> Result<(), Error> {
        let path = format!("/portfolios/{}/addItem", portfolio_gid);
        let request = AddItemRequest {
            data: AddItemData {
                item: item_gid.to_string(),
                insert_before: insert_before.map(String::from),
                insert_after: insert_after.map(String::from),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove an item from a portfolio.
    pub async fn remove_item(&self, portfolio_gid: &str, item_gid: &str) -> Result<(), Error> {
        let path = format!("/portfolios/{}/removeItem", portfolio_gid);
        let request = RemoveItemRequest {
            data: RemoveItemData {
                item: item_gid.to_string(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add members to a portfolio.
    pub async fn add_members(
        &self,
        portfolio_gid: &str,
        member_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/portfolios/{}/addMembers", portfolio_gid);
        let request = AddMembersRequest {
            data: AddMembersData {
                members: member_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove members from a portfolio.
    pub async fn remove_members(
        &self,
        portfolio_gid: &str,
        member_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/portfolios/{}/removeMembers", portfolio_gid);
        let request = RemoveMembersRequest {
            data: RemoveMembersData {
                members: member_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Get a portfolio with its items recursively expanded.
    ///
    /// The `max_depth` parameter controls how many levels of children to include:
    /// - `None` - Unlimited depth (expand all nested portfolios)
    /// - `Some(0)` - Just the portfolio metadata, no items
    /// - `Some(1)` - Portfolio + immediate children only
    /// - `Some(2)` - Portfolio + children + grandchildren
    /// - etc.
    ///
    /// Note: Tasks are never included. Use `projects().tasks()` separately if needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    ///
    /// // Get portfolio with all nested items
    /// let portfolio = client.portfolios().recursive("port123", None).await?;
    ///
    /// // Get portfolio with only direct children
    /// let portfolio = client.portfolios().recursive("port123", Some(1)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recursive(
        &self,
        gid: &str,
        max_depth: Option<usize>,
    ) -> Result<PortfolioWithItems, Error> {
        self.fetch_with_depth(gid, max_depth, 0).await
    }

    /// Internal recursive helper for fetching portfolios.
    fn fetch_with_depth<'b>(
        &'b self,
        gid: &'b str,
        max_depth: Option<usize>,
        current_depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PortfolioWithItems, Error>> + Send + 'b>,
    > {
        Box::pin(async move {
            let portfolio = self.get(gid).await?;

            // Check if we should fetch items at this depth
            let should_fetch_items = match max_depth {
                None => true,                     // Unlimited depth
                Some(max) => current_depth < max, // Only if we haven't reached max
            };

            if !should_fetch_items {
                return Ok(PortfolioWithItems {
                    portfolio,
                    items: Vec::new(),
                });
            }

            let item_refs = self.items(gid).await?;
            let mut items = Vec::new();

            for item_ref in item_refs {
                let expanded = match item_ref.resource_type.as_str() {
                    "project" => {
                        let project = self.client.projects().get_full(&item_ref.gid).await?;
                        PortfolioItemExpanded::Project(Box::new(project))
                    }
                    "portfolio" => {
                        let nested = self
                            .fetch_with_depth(&item_ref.gid, max_depth, current_depth + 1)
                            .await?;
                        PortfolioItemExpanded::Portfolio(Box::new(nested))
                    }
                    _ => continue, // Unknown type, skip
                };
                items.push(expanded);
            }

            Ok(PortfolioWithItems { portfolio, items })
        })
    }
}

/// A portfolio with its nested items expanded.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PortfolioWithItems {
    /// The portfolio details.
    #[serde(flatten)]
    pub portfolio: Portfolio,
    /// The items in the portfolio.
    pub items: Vec<PortfolioItemExpanded>,
}

/// An expanded portfolio item with full details.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "resource_type", rename_all = "snake_case")]
pub enum PortfolioItemExpanded {
    /// A project in the portfolio.
    Project(Box<Project>),
    /// A nested portfolio with its items.
    Portfolio(Box<PortfolioWithItems>),
}

impl Client {
    /// Access the portfolios API.
    pub fn portfolios(&self) -> PortfoliosApi<'_> {
        PortfoliosApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_create_portfolio() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "newport",
                    "name": "New Portfolio",
                    "public": true
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let portfolio = client
            .portfolios()
            .create(CreatePortfolioData {
                name: "New Portfolio".to_string(),
                workspace: "ws123".to_string(),
                public: Some(true),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(portfolio.gid, "newport");
        assert_eq!(portfolio.name, "New Portfolio");
    }

    #[tokio::test]
    async fn test_update_portfolio() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/portfolios/port123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "port123",
                    "name": "Updated Portfolio",
                    "color": "blue"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let portfolio = client
            .portfolios()
            .update(
                "port123",
                UpdatePortfolioData {
                    name: Some("Updated Portfolio".to_string()),
                    color: Some("blue".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(portfolio.name, "Updated Portfolio");
    }

    #[tokio::test]
    async fn test_add_item() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios/port123/addItem"))
            .and(body_json(serde_json::json!({
                "data": {"item": "proj456"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.portfolios().add_item("port123", "proj456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_item() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios/port123/removeItem"))
            .and(body_json(serde_json::json!({
                "data": {"item": "proj456"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.portfolios().remove_item("port123", "proj456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_members() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios/port123/addMembers"))
            .and(body_json(serde_json::json!({
                "data": {"members": ["user1", "user2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .portfolios()
            .add_members("port123", &["user1", "user2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_members() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios/port123/removeMembers"))
            .and(body_json(serde_json::json!({
                "data": {"members": ["user1"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .portfolios()
            .remove_members("port123", &["user1"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_portfolio() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/portfolios/port123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.portfolios().delete("port123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_item_with_position() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/portfolios/port123/addItem"))
            .and(body_json(serde_json::json!({
                "data": {
                    "item": "proj456",
                    "insert_before": "proj789"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .portfolios()
            .add_item_with_position("port123", "proj456", Some("proj789"), None)
            .await;

        assert!(result.is_ok());
    }
}
