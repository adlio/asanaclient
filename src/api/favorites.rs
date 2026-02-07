//! High-level API for fetching user favorites with full context.

use crate::api::portfolios::PortfolioWithItems;
use crate::types::{
    extract_status_field, ExtractedStatus, FavoriteItem, Project, StatusExtractionOptions, Task,
};
use crate::{Client, Error};

/// Options for fetching favorites.
#[derive(Debug, Clone)]
pub struct FetchFavoritesOptions {
    /// Whether to fetch full project details (default: true).
    pub include_projects: bool,
    /// Whether to fetch full portfolio details (default: true).
    pub include_portfolios: bool,
    /// Maximum depth for recursive portfolio fetching.
    /// - `None` - Unlimited depth
    /// - `Some(0)` - No expansion (default)
    /// - `Some(n)` - Fetch n levels deep
    pub portfolio_depth: Option<usize>,
    /// Whether to fetch tasks for projects (default: false).
    pub include_project_tasks: bool,
    /// Options for extracting status from custom fields.
    pub status_extraction: Option<StatusExtractionOptions>,
}

impl Default for FetchFavoritesOptions {
    fn default() -> Self {
        Self {
            include_projects: true,
            include_portfolios: true,
            portfolio_depth: Some(0),
            include_project_tasks: false,
            status_extraction: None,
        }
    }
}

impl FetchFavoritesOptions {
    /// Create default options with projects and portfolios enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Include tasks for projects.
    pub fn with_tasks(mut self) -> Self {
        self.include_project_tasks = true;
        self
    }

    /// Set portfolio recursion depth.
    /// - `None` - Unlimited depth
    /// - `Some(0)` - No expansion
    /// - `Some(n)` - Fetch n levels deep
    pub fn with_portfolio_depth(mut self, depth: Option<usize>) -> Self {
        self.portfolio_depth = depth;
        self
    }

    /// Set status extraction options.
    pub fn with_status_extraction(mut self, options: StatusExtractionOptions) -> Self {
        self.status_extraction = Some(options);
        self
    }
}

/// A project with additional context.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectWithContext {
    /// The project details.
    #[serde(flatten)]
    pub project: Project,
    /// Tasks in the project (if fetched).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<Task>,
    /// Extracted status from custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted_status: Option<ExtractedStatus>,
}

/// Complete favorites data for a workspace.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FavoritesData {
    /// Favorited projects with context.
    pub projects: Vec<ProjectWithContext>,
    /// Favorited portfolios with items.
    pub portfolios: Vec<PortfolioWithItems>,
    /// Items that failed to fetch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FetchError>,
}

/// An error that occurred while fetching a favorite.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FetchError {
    /// The item that failed to fetch.
    pub item: FavoriteItem,
    /// The error message.
    pub error: String,
}

impl Client {
    /// Fetch all favorites for a workspace with full context.
    ///
    /// This is a high-level method that fetches favorites and expands them
    /// with full details including nested portfolios and optionally tasks.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # use asanaclient::api::favorites::FetchFavoritesOptions;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let options = FetchFavoritesOptions::new().with_tasks();
    /// let favorites = client.favorites("workspace_gid", options).await?;
    ///
    /// println!("Projects: {}", favorites.projects.len());
    /// println!("Portfolios: {}", favorites.portfolios.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn favorites(
        &self,
        workspace_gid: &str,
        options: FetchFavoritesOptions,
    ) -> Result<FavoritesData, Error> {
        // Get favorites list
        let favorites_list = self.users().favorites(workspace_gid).await?;

        let mut projects = Vec::new();
        let mut portfolios = Vec::new();
        let mut errors = Vec::new();

        for item in favorites_list {
            match item.resource_type.as_str() {
                "project" if options.include_projects => {
                    match self.fetch_project_with_context(&item.gid, &options).await {
                        Ok(project) => projects.push(project),
                        Err(e) => errors.push(FetchError {
                            item,
                            error: e.to_string(),
                        }),
                    }
                }
                "portfolio" if options.include_portfolios => {
                    match self
                        .portfolios()
                        .recursive(&item.gid, options.portfolio_depth)
                        .await
                    {
                        Ok(portfolio) => portfolios.push(portfolio),
                        Err(e) => errors.push(FetchError {
                            item,
                            error: e.to_string(),
                        }),
                    }
                }
                _ => {}
            }
        }

        Ok(FavoritesData {
            projects,
            portfolios,
            errors,
        })
    }

    /// Fetch a project with additional context.
    async fn fetch_project_with_context(
        &self,
        gid: &str,
        options: &FetchFavoritesOptions,
    ) -> Result<ProjectWithContext, Error> {
        let project = self.projects().get_full(gid).await?;

        let tasks = if options.include_project_tasks {
            self.projects().tasks(gid).await?
        } else {
            Vec::new()
        };

        // Note: custom_field_settings on projects contain definitions, not values
        // Status extraction would typically be done on task custom_fields
        let extracted_status = None;

        Ok(ProjectWithContext {
            project,
            tasks,
            extracted_status,
        })
    }
}

/// Extract status from a task's custom fields.
pub fn extract_task_status(
    task: &Task,
    options: Option<StatusExtractionOptions>,
) -> Option<ExtractedStatus> {
    extract_status_field(&task.custom_fields, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_favorites_options_default() {
        let opts = FetchFavoritesOptions::new();
        assert!(opts.include_projects);
        assert!(opts.include_portfolios);
        assert_eq!(opts.portfolio_depth, Some(0));
        assert!(!opts.include_project_tasks);
    }

    #[test]
    fn test_fetch_favorites_options_builder() {
        let opts = FetchFavoritesOptions::new()
            .with_tasks()
            .with_portfolio_depth(Some(5));

        assert!(opts.include_project_tasks);
        assert_eq!(opts.portfolio_depth, Some(5));
    }

    #[test]
    fn test_portfolio_depth_unlimited() {
        let opts = FetchFavoritesOptions::new().with_portfolio_depth(None);
        assert_eq!(opts.portfolio_depth, None);
    }
}
