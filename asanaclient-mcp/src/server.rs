//! MCP server implementation for Asana.

use asanaclient::{Client, FavoriteItem, Project};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorData as McpError, Implementation, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use rmcp::{schemars, tool, tool_handler, tool_router, ServerHandler};
use serde::{Deserialize, Serialize};

/// MCP server for Asana operations.
#[derive(Debug, Clone)]
pub struct AsanaServer {
    client: Client,
    tool_router: ToolRouter<AsanaServer>,
}

/// Parameters for listing workspaces.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListWorkspacesParams {}

/// Parameters for getting user favorites.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFavoritesParams {
    /// The GID of the workspace to get favorites from.
    pub workspace_gid: String,
    /// Whether to include full project details (default: true).
    #[serde(default = "default_true")]
    pub include_projects: bool,
    /// Whether to include full portfolio details (default: true).
    #[serde(default = "default_true")]
    pub include_portfolios: bool,
    /// Depth for recursive portfolio fetching. Use -1 for unlimited, 0 for no items,
    /// or a positive number for that many levels (default: 3).
    #[serde(default = "default_depth")]
    pub portfolio_depth: i32,
}

fn default_true() -> bool {
    true
}

fn default_depth() -> i32 {
    3
}

/// Convert a depth parameter to `Option<usize>`.
/// -1 means unlimited (None), 0+ means that many levels.
fn depth_to_option(depth: i32) -> Option<usize> {
    if depth < 0 {
        None
    } else {
        Some(depth as usize)
    }
}

/// Parameters for getting a project.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetProjectParams {
    /// The GID of the project to retrieve.
    pub project_gid: String,
}

/// Parameters for getting a portfolio.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPortfolioParams {
    /// The GID of the portfolio to retrieve.
    pub portfolio_gid: String,
    /// Depth for recursive portfolio fetching. Use -1 for unlimited, 0 for just
    /// the portfolio metadata, or a positive number for that many levels of
    /// children (default: 3).
    #[serde(default = "default_depth")]
    pub depth: i32,
}

/// Parameters for getting a task.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskParams {
    /// The GID of the task to retrieve.
    pub task_gid: String,
    /// Whether to include subtasks (default: true).
    #[serde(default = "default_true")]
    pub include_subtasks: bool,
    /// Whether to include dependencies and dependents (default: true).
    #[serde(default = "default_true")]
    pub include_dependencies: bool,
    /// Whether to include comments (default: true).
    #[serde(default = "default_true")]
    pub include_comments: bool,
}

/// Parameters for getting tasks recursively from a project or portfolio.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTasksRecursiveParams {
    /// The GID of the project or portfolio to get tasks from.
    /// The resource type is auto-detected.
    pub gid: String,
    /// Depth for subtask expansion. Use -1 for unlimited, 0 for no subtasks
    /// (top-level tasks only), or a positive number for that many levels
    /// of subtasks (default: 0).
    #[serde(default)]
    pub subtask_depth: i32,
    /// Depth for portfolio traversal (only applies when GID is a portfolio).
    /// Use -1 for unlimited, 0 for direct child projects only, or a positive
    /// number for that many levels of nested portfolios (default: 3).
    #[serde(default = "default_depth")]
    pub portfolio_depth: i32,
}

/// Response containing user favorites with full details.
#[derive(Debug, Serialize)]
struct FavoritesResponse {
    /// The favorited projects.
    projects: Vec<Project>,
    /// The favorited portfolios with their items.
    portfolios: Vec<asanaclient::PortfolioWithItems>,
    /// Items that couldn't be fetched.
    errors: Vec<FavoriteError>,
}

/// An error fetching a favorite item.
#[derive(Debug, Serialize)]
struct FavoriteError {
    /// The item that failed.
    item: FavoriteItem,
    /// The error message.
    error: String,
}

/// Helper function to convert errors to MCP errors.
fn to_mcp_error(context: &str, error: impl std::fmt::Display) -> McpError {
    McpError::new(
        rmcp::model::ErrorCode::INTERNAL_ERROR,
        format!("{}: {}", context, error),
        None,
    )
}

/// Helper function to serialize response to JSON and wrap in CallToolResult.
fn json_response<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| to_mcp_error("Failed to serialize response", e))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

#[tool_router]
impl AsanaServer {
    /// Create a new Asana MCP server.
    ///
    /// Reads the `ASANA_TOKEN` environment variable for authentication.
    pub fn new() -> Result<Self, asanaclient::Error> {
        let client = Client::from_env()?;
        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }

    /// Create a server with a custom client (for testing).
    #[cfg(test)]
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    /// List all workspaces accessible to the authenticated user.
    #[tool(description = "List all workspaces accessible to the authenticated user")]
    async fn asana_list_workspaces(
        &self,
        _params: Parameters<ListWorkspacesParams>,
    ) -> Result<CallToolResult, McpError> {
        let workspaces = self
            .client
            .workspaces()
            .list()
            .await
            .map_err(|e| to_mcp_error("Failed to list workspaces", e))?;

        json_response(&workspaces)
    }

    /// Get the current user's favorites in a workspace with full details.
    #[tool(
        description = "Get favorited projects and portfolios for the current user in a workspace. Returns full details including status updates and nested portfolio contents."
    )]
    async fn asana_get_favorites(
        &self,
        params: Parameters<GetFavoritesParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let depth = depth_to_option(params.portfolio_depth);

        let favorites = self
            .client
            .users()
            .favorites(&params.workspace_gid)
            .await
            .map_err(|e| to_mcp_error("Failed to get favorites", e))?;

        let mut projects = Vec::new();
        let mut portfolios = Vec::new();
        let mut errors = Vec::new();

        for item in favorites {
            match item.resource_type.as_str() {
                "project" if params.include_projects => {
                    match self.client.projects().get_full(&item.gid).await {
                        Ok(project) => projects.push(project),
                        Err(e) => errors.push(FavoriteError {
                            item,
                            error: e.to_string(),
                        }),
                    }
                }
                "portfolio" if params.include_portfolios => {
                    match self.client.get_portfolio_recursive(&item.gid, depth).await {
                        Ok(portfolio) => portfolios.push(portfolio),
                        Err(e) => errors.push(FavoriteError {
                            item,
                            error: e.to_string(),
                        }),
                    }
                }
                _ => {}
            }
        }

        json_response(&FavoritesResponse {
            projects,
            portfolios,
            errors,
        })
    }

    /// Get a single project by its GID.
    #[tool(description = "Get a project by its GID with full details including status updates")]
    async fn asana_get_project(
        &self,
        params: Parameters<GetProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let project = self
            .client
            .projects()
            .get_full(&params.0.project_gid)
            .await
            .map_err(|e| to_mcp_error("Failed to get project", e))?;

        json_response(&project)
    }

    /// Get a portfolio by its GID with recursive nesting.
    #[tool(
        description = "Get a portfolio by its GID with all nested items (projects and sub-portfolios) expanded recursively"
    )]
    async fn asana_get_portfolio(
        &self,
        params: Parameters<GetPortfolioParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let depth = depth_to_option(params.depth);

        let portfolio = self
            .client
            .get_portfolio_recursive(&params.portfolio_gid, depth)
            .await
            .map_err(|e| to_mcp_error("Failed to get portfolio", e))?;

        json_response(&portfolio)
    }

    /// Get a task by its GID with optional context.
    #[tool(
        description = "Get a task by its GID with full details including subtasks, dependencies, and comments"
    )]
    async fn asana_get_task(
        &self,
        params: Parameters<GetTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let task = self
            .client
            .get_task_with_context(
                &params.task_gid,
                params.include_subtasks,
                params.include_dependencies,
                params.include_comments,
            )
            .await
            .map_err(|e| to_mcp_error("Failed to get task", e))?;

        json_response(&task)
    }

    /// Get all tasks recursively from a project or portfolio.
    #[tool(
        description = "Get all tasks from a project or portfolio. Auto-detects resource type. \
            For portfolios, recursively finds all projects and returns their tasks. \
            Each task includes ALL its projects (not just ones in the queried hierarchy). \
            Use subtask_depth to control subtask expansion: -1 for unlimited, 0 for none, N for N levels. \
            Use portfolio_depth to control how deep to search nested portfolios: -1 for unlimited, 0 for direct projects only, N for N levels (default: 3)."
    )]
    async fn asana_get_tasks_recursive(
        &self,
        params: Parameters<GetTasksRecursiveParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let subtask_depth = if params.subtask_depth < 0 {
            None
        } else {
            Some(params.subtask_depth)
        };
        let portfolio_depth = Some(params.portfolio_depth);

        let tasks = self
            .client
            .get_tasks_recursive(&params.gid, subtask_depth, portfolio_depth)
            .await
            .map_err(|e| to_mcp_error("Failed to get tasks", e))?;

        json_response(&tasks)
    }
}

#[tool_handler]
impl ServerHandler for AsanaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "asanaclient-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Asana MCP server providing tools for interacting with Asana tasks, \
                 projects, and portfolios. Authenticate with ASANA_TOKEN environment variable."
                    .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    /// Matcher for requests without an "offset" query parameter.
    struct NoOffset;

    impl Match for NoOffset {
        fn matches(&self, request: &Request) -> bool {
            !request.url.query_pairs().any(|(k, _)| k == "offset")
        }
    }

    /// Create a test server with client pointing to mock.
    fn test_server(mock_uri: &str) -> AsanaServer {
        let client = Client::new("test-token").unwrap().with_base_url(mock_uri);
        AsanaServer::with_client(client)
    }

    /// Extract text content from a CallToolResult.
    fn get_response_text(result: &CallToolResult) -> &str {
        &result.content[0]
            .as_text()
            .expect("Expected text content")
            .text
    }

    // ========== list_workspaces tests ==========

    #[tokio::test]
    async fn test_list_workspaces_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/workspaces"))
            .and(NoOffset)
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "123", "name": "My Workspace", "is_organization": true},
                    {"gid": "456", "name": "Another Workspace", "is_organization": false}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(ListWorkspacesParams {});

        let result = server.asana_list_workspaces(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("My Workspace"));
        assert!(text.contains("Another Workspace"));
        assert!(text.contains("\"gid\": \"123\""));
    }

    #[tokio::test]
    async fn test_list_workspaces_empty() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/workspaces"))
            .and(NoOffset)
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let result = server
            .asana_list_workspaces(Parameters(ListWorkspacesParams {}))
            .await
            .unwrap();

        assert!(get_response_text(&result).contains("[]"));
    }

    // ========== get_project tests ==========

    #[tokio::test]
    async fn test_get_project_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/projects/proj123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "proj123",
                    "name": "Test Project",
                    "archived": false,
                    "public": true,
                    "notes": "Project description"
                }
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetProjectParams {
            project_gid: "proj123".to_string(),
        });

        let result = server.asana_get_project(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Test Project"));
        assert!(text.contains("proj123"));
    }

    #[tokio::test]
    async fn test_get_project_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/projects/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetProjectParams {
            project_gid: "missing".to_string(),
        });

        let result = server.asana_get_project(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Failed to get project"));
    }

    // ========== get_task tests ==========

    #[tokio::test]
    async fn test_get_task_success() {
        let mock_server = MockServer::start().await;

        // Task endpoint
        Mock::given(method("GET"))
            .and(path("/tasks/task123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "task123",
                    "name": "Test Task",
                    "completed": false,
                    "notes": "Task notes"
                }
            })))
            .mount(&mock_server)
            .await;

        // Subtasks endpoint
        Mock::given(method("GET"))
            .and(path("/tasks/task123/subtasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"gid": "sub1", "name": "Subtask 1", "completed": false}],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        // Dependencies endpoint
        Mock::given(method("GET"))
            .and(path("/tasks/task123/dependencies"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        // Dependents endpoint
        Mock::given(method("GET"))
            .and(path("/tasks/task123/dependents"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        // Stories endpoint
        Mock::given(method("GET"))
            .and(path("/tasks/task123/stories"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "story1", "resource_subtype": "comment_added", "text": "A comment"}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetTaskParams {
            task_gid: "task123".to_string(),
            include_subtasks: true,
            include_dependencies: true,
            include_comments: true,
        });

        let result = server.asana_get_task(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Test Task"));
        assert!(text.contains("task123"));
        assert!(text.contains("Subtask 1"));
        assert!(text.contains("A comment"));
    }

    #[tokio::test]
    async fn test_get_task_minimal() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tasks/task456"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "task456",
                    "name": "Minimal Task",
                    "completed": true
                }
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetTaskParams {
            task_gid: "task456".to_string(),
            include_subtasks: false,
            include_dependencies: false,
            include_comments: false,
        });

        let result = server.asana_get_task(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Minimal Task"));
        assert!(text.contains("task456"));
    }

    // ========== get_portfolio tests ==========

    #[tokio::test]
    async fn test_get_portfolio_success() {
        let mock_server = MockServer::start().await;

        // Portfolio endpoint
        Mock::given(method("GET"))
            .and(path("/portfolios/port123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "port123",
                    "name": "Test Portfolio",
                    "public": true
                }
            })))
            .mount(&mock_server)
            .await;

        // Portfolio items endpoint
        Mock::given(method("GET"))
            .and(path("/portfolios/port123/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "proj1", "resource_type": "project", "name": "Project in Portfolio"}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        // Project endpoint for nested project
        Mock::given(method("GET"))
            .and(path("/projects/proj1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "proj1",
                    "name": "Project in Portfolio",
                    "archived": false
                }
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetPortfolioParams {
            portfolio_gid: "port123".to_string(),
            depth: 1,
        });

        let result = server.asana_get_portfolio(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Test Portfolio"));
        assert!(text.contains("Project in Portfolio"));
    }

    // ========== get_favorites tests ==========

    #[tokio::test]
    async fn test_get_favorites_projects_only() {
        let mock_server = MockServer::start().await;

        // Favorites endpoint
        Mock::given(method("GET"))
            .and(path("/users/me/favorites"))
            .and(query_param("workspace", "ws123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "fav1", "resource_type": "project", "name": "Favorite Project"}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        // Project endpoint
        Mock::given(method("GET"))
            .and(path("/projects/fav1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "fav1",
                    "name": "Favorite Project",
                    "archived": false
                }
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetFavoritesParams {
            workspace_gid: "ws123".to_string(),
            include_projects: true,
            include_portfolios: false,
            portfolio_depth: 1,
        });

        let result = server.asana_get_favorites(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Favorite Project"));
        assert!(text.contains("\"projects\""));
    }

    #[tokio::test]
    async fn test_get_favorites_empty() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/users/me/favorites"))
            .and(query_param("workspace", "ws456"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let params = Parameters(GetFavoritesParams {
            workspace_gid: "ws456".to_string(),
            include_projects: true,
            include_portfolios: true,
            portfolio_depth: 3,
        });

        let result = server.asana_get_favorites(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("\"projects\": []"));
        assert!(text.contains("\"portfolios\": []"));
    }

    // ========== ServerHandler tests ==========

    #[test]
    fn test_server_info() {
        // Can't easily create a server without a real token for this test,
        // but we can test the handler implementation is correct
        let client = Client::new("test").unwrap();
        let server = AsanaServer::with_client(client);
        let info = server.get_info();

        assert_eq!(info.server_info.name, "asanaclient-mcp");
        assert!(info.capabilities.tools.is_some());
    }
}
