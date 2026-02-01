//! Portfolio API endpoints.

use crate::types::{Portfolio, PortfolioItemRef, Project, StatusUpdate};
use crate::{Client, Error};

/// Fields to request for a basic portfolio fetch.
pub const PORTFOLIO_FIELDS: &str = "gid,name,color,owner,owner.name,workspace,\
    current_status_update,current_status_update.status_type,current_status_update.title,\
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
    pub async fn get_portfolio_recursive(
        &self,
        gid: &str,
        max_depth: Option<usize>,
    ) -> Result<PortfolioWithItems, Error> {
        self.fetch_portfolio_with_depth(gid, max_depth, 0).await
    }

    /// Internal recursive helper for fetching portfolios.
    fn fetch_portfolio_with_depth<'a>(
        &'a self,
        gid: &'a str,
        max_depth: Option<usize>,
        current_depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PortfolioWithItems, Error>> + Send + 'a>,
    > {
        Box::pin(async move {
            let portfolio = self.portfolios().get(gid).await?;

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

            let item_refs = self.portfolios().items(gid).await?;
            let mut items = Vec::new();

            for item_ref in item_refs {
                let expanded = match item_ref.resource_type.as_str() {
                    "project" => {
                        let project = self.projects().get(&item_ref.gid).await?;
                        PortfolioItemExpanded::Project(Box::new(project))
                    }
                    "portfolio" => {
                        let nested = self
                            .fetch_portfolio_with_depth(&item_ref.gid, max_depth, current_depth + 1)
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
