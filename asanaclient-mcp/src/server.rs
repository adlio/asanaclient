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

/// Parameters for listing workspaces (no parameters needed).
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkspacesParams {}

/// The type of resource to fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// A project (gid = project GID)
    Project,
    /// A portfolio with nested items (gid = portfolio GID)
    Portfolio,
    /// A task with optional context (gid = task GID)
    Task,
    /// User's favorited projects and portfolios (gid = workspace GID)
    Favorites,
    /// Tasks in a project or portfolio (gid = project/portfolio GID)
    Tasks,
    /// Subtasks of a task (gid = parent task GID)
    Subtasks,
    /// Comments on a task (gid = task GID)
    Comments,
    /// Status updates for a project or portfolio (gid = project/portfolio GID)
    StatusUpdates,
}

/// Parameters for the universal get tool.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetParams {
    /// The type of resource to fetch.
    pub resource_type: ResourceType,

    /// The GID of the resource or its parent (meaning depends on resource_type).
    /// - project/portfolio/task: the resource's GID
    /// - favorites: the workspace GID
    /// - tasks: the project or portfolio GID
    /// - subtasks/comments: the task GID
    /// - status_updates: the project or portfolio GID
    pub gid: String,

    // === Depth controls ===
    /// Depth for recursive fetching (portfolios, tasks).
    /// -1 = unlimited, 0 = no expansion, N = N levels.
    /// Default: 3 for portfolios, 0 for subtasks.
    #[serde(default)]
    pub depth: Option<i32>,

    /// Depth for subtask expansion when fetching tasks.
    /// -1 = unlimited, 0 = no subtasks, N = N levels. Default: 0.
    #[serde(default)]
    pub subtask_depth: Option<i32>,

    // === Include flags (for tasks) ===
    /// Include subtask references when fetching a task. Default: true.
    #[serde(default)]
    pub include_subtasks: Option<bool>,

    /// Include dependency/dependent references when fetching a task. Default: true.
    #[serde(default)]
    pub include_dependencies: Option<bool>,

    /// Include comments when fetching a task. Default: true.
    #[serde(default)]
    pub include_comments: Option<bool>,

    // === Include flags (for favorites) ===
    /// Include projects when fetching favorites. Default: true.
    #[serde(default)]
    pub include_projects: Option<bool>,

    /// Include portfolios when fetching favorites. Default: true.
    #[serde(default)]
    pub include_portfolios: Option<bool>,
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
    #[tool(description = "List all Asana workspaces accessible to the authenticated user")]
    async fn asana_workspaces(
        &self,
        _params: Parameters<WorkspacesParams>,
    ) -> Result<CallToolResult, McpError> {
        let workspaces = self
            .client
            .workspaces()
            .list()
            .await
            .map_err(|e| to_mcp_error("Failed to list workspaces", e))?;

        json_response(&workspaces)
    }

    /// Universal get tool for fetching Asana resources.
    #[tool(description = "Get any Asana resource by type and GID. Supports:\n\
            - project: Get a project (gid = project GID)\n\
            - portfolio: Get a portfolio with nested items (gid = portfolio GID, use depth to control recursion)\n\
            - task: Get a task with context (gid = task GID, use include_* flags)\n\
            - favorites: Get user's favorites (gid = workspace GID)\n\
            - tasks: Get all tasks from a project/portfolio (gid = project/portfolio GID, use subtask_depth)\n\
            - subtasks: Get subtasks of a task (gid = task GID)\n\
            - comments: Get comments on a task (gid = task GID)\n\
            - status_updates: Get status history (gid = project/portfolio GID)\n\n\
            Depth parameters: -1 = unlimited, 0 = none, N = N levels")]
    async fn asana_get(&self, params: Parameters<GetParams>) -> Result<CallToolResult, McpError> {
        let p = params.0;

        match p.resource_type {
            ResourceType::Project => {
                let project = self
                    .client
                    .projects()
                    .get_full(&p.gid)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get project", e))?;
                json_response(&project)
            }

            ResourceType::Portfolio => {
                let depth = depth_to_option(p.depth.unwrap_or(default_depth()));
                let portfolio = self
                    .client
                    .get_portfolio_recursive(&p.gid, depth)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get portfolio", e))?;
                json_response(&portfolio)
            }

            ResourceType::Task => {
                let task = self
                    .client
                    .get_task_with_context(
                        &p.gid,
                        p.include_subtasks.unwrap_or(true),
                        p.include_dependencies.unwrap_or(true),
                        p.include_comments.unwrap_or(true),
                    )
                    .await
                    .map_err(|e| to_mcp_error("Failed to get task", e))?;
                json_response(&task)
            }

            ResourceType::Favorites => {
                let depth = depth_to_option(p.depth.unwrap_or(default_depth()));
                let include_projects = p.include_projects.unwrap_or(true);
                let include_portfolios = p.include_portfolios.unwrap_or(true);

                let favorites = self
                    .client
                    .users()
                    .favorites(&p.gid)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get favorites", e))?;

                let mut projects = Vec::new();
                let mut portfolios = Vec::new();
                let mut errors = Vec::new();

                for item in favorites {
                    match item.resource_type.as_str() {
                        "project" if include_projects => {
                            match self.client.projects().get_full(&item.gid).await {
                                Ok(project) => projects.push(project),
                                Err(e) => errors.push(FavoriteError {
                                    item,
                                    error: e.to_string(),
                                }),
                            }
                        }
                        "portfolio" if include_portfolios => {
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

            ResourceType::Tasks => {
                let subtask_depth = p
                    .subtask_depth
                    .map(|d| if d < 0 { None } else { Some(d) })
                    .unwrap_or(Some(0));
                let portfolio_depth = Some(p.depth.unwrap_or(default_depth()));

                let tasks = self
                    .client
                    .get_tasks_recursive(&p.gid, subtask_depth, portfolio_depth)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get tasks", e))?;
                json_response(&tasks)
            }

            ResourceType::Subtasks => {
                let subtasks = self
                    .client
                    .tasks()
                    .subtasks(&p.gid)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get subtasks", e))?;
                json_response(&subtasks)
            }

            ResourceType::Comments => {
                let comments = self
                    .client
                    .tasks()
                    .comments(&p.gid)
                    .await
                    .map_err(|e| to_mcp_error("Failed to get comments", e))?;
                json_response(&comments)
            }

            ResourceType::StatusUpdates => {
                // Try as project first, then as portfolio
                let updates = match self.client.projects().status_updates(&p.gid).await {
                    Ok(updates) => updates,
                    Err(asanaclient::Error::NotFound(_)) => self
                        .client
                        .portfolios()
                        .status_updates(&p.gid)
                        .await
                        .map_err(|e| to_mcp_error("Failed to get status updates", e))?,
                    Err(e) => return Err(to_mcp_error("Failed to get status updates", e)),
                };
                json_response(&updates)
            }
        }
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

    /// Helper to create GetParams with defaults.
    fn get_params(resource_type: ResourceType, gid: &str) -> Parameters<GetParams> {
        Parameters(GetParams {
            resource_type,
            gid: gid.to_string(),
            depth: None,
            subtask_depth: None,
            include_subtasks: None,
            include_dependencies: None,
            include_comments: None,
            include_projects: None,
            include_portfolios: None,
        })
    }

    // ========== asana_workspaces tests ==========

    #[tokio::test]
    async fn test_workspaces_success() {
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
        let result = server
            .asana_workspaces(Parameters(WorkspacesParams {}))
            .await
            .unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("My Workspace"));
        assert!(text.contains("Another Workspace"));
        assert!(text.contains("\"gid\": \"123\""));
    }

    #[tokio::test]
    async fn test_workspaces_empty() {
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
            .asana_workspaces(Parameters(WorkspacesParams {}))
            .await
            .unwrap();

        assert!(get_response_text(&result).contains("[]"));
    }

    // ========== asana_get project tests ==========

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
        let result = server
            .asana_get(get_params(ResourceType::Project, "proj123"))
            .await
            .unwrap();
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
        let result = server
            .asana_get(get_params(ResourceType::Project, "missing"))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Failed to get project"));
    }

    // ========== asana_get task tests ==========

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
        let result = server
            .asana_get(get_params(ResourceType::Task, "task123"))
            .await
            .unwrap();
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
        let params = Parameters(GetParams {
            resource_type: ResourceType::Task,
            gid: "task456".to_string(),
            depth: None,
            subtask_depth: None,
            include_subtasks: Some(false),
            include_dependencies: Some(false),
            include_comments: Some(false),
            include_projects: None,
            include_portfolios: None,
        });

        let result = server.asana_get(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Minimal Task"));
        assert!(text.contains("task456"));
    }

    // ========== asana_get portfolio tests ==========

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
        let params = Parameters(GetParams {
            resource_type: ResourceType::Portfolio,
            gid: "port123".to_string(),
            depth: Some(1),
            subtask_depth: None,
            include_subtasks: None,
            include_dependencies: None,
            include_comments: None,
            include_projects: None,
            include_portfolios: None,
        });

        let result = server.asana_get(params).await.unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Test Portfolio"));
        assert!(text.contains("Project in Portfolio"));
    }

    // ========== asana_get favorites tests ==========

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
        let params = Parameters(GetParams {
            resource_type: ResourceType::Favorites,
            gid: "ws123".to_string(),
            depth: Some(1),
            subtask_depth: None,
            include_subtasks: None,
            include_dependencies: None,
            include_comments: None,
            include_projects: Some(true),
            include_portfolios: Some(false),
        });

        let result = server.asana_get(params).await.unwrap();
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
        let result = server
            .asana_get(get_params(ResourceType::Favorites, "ws456"))
            .await
            .unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("\"projects\": []"));
        assert!(text.contains("\"portfolios\": []"));
    }

    // ========== asana_get comments tests ==========

    #[tokio::test]
    async fn test_get_comments_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tasks/task123/stories"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "story1", "resource_subtype": "comment_added", "text": "First comment"},
                    {"gid": "story2", "resource_subtype": "assigned", "text": "System message"},
                    {"gid": "story3", "resource_subtype": "comment_added", "text": "Second comment"}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let result = server
            .asana_get(get_params(ResourceType::Comments, "task123"))
            .await
            .unwrap();
        let text = get_response_text(&result);

        // Should only include comments, not system messages
        assert!(text.contains("First comment"));
        assert!(text.contains("Second comment"));
    }

    // ========== asana_get subtasks tests ==========

    #[tokio::test]
    async fn test_get_subtasks_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tasks/task123/subtasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "sub1", "name": "Subtask 1", "completed": false},
                    {"gid": "sub2", "name": "Subtask 2", "completed": true}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let result = server
            .asana_get(get_params(ResourceType::Subtasks, "task123"))
            .await
            .unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Subtask 1"));
        assert!(text.contains("Subtask 2"));
    }

    // ========== asana_get status_updates tests ==========

    #[tokio::test]
    async fn test_get_status_updates_project() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/projects/proj123/status_updates"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "status1", "title": "On Track", "status_type": "on_track", "text": "All good"}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let result = server
            .asana_get(get_params(ResourceType::StatusUpdates, "proj123"))
            .await
            .unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("On Track"));
        assert!(text.contains("All good"));
    }

    // ========== asana_get tasks (recursive) tests ==========

    #[tokio::test]
    async fn test_get_tasks_from_project() {
        let mock_server = MockServer::start().await;

        // First try as project (succeeds)
        Mock::given(method("GET"))
            .and(path("/projects/proj123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"gid": "proj123", "name": "Test Project"}
            })))
            .mount(&mock_server)
            .await;

        // Project tasks endpoint
        Mock::given(method("GET"))
            .and(path("/projects/proj123/tasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "task1", "name": "Task 1", "completed": false, "num_subtasks": 0},
                    {"gid": "task2", "name": "Task 2", "completed": true, "num_subtasks": 0}
                ],
                "next_page": null
            })))
            .mount(&mock_server)
            .await;

        let server = test_server(&mock_server.uri());
        let result = server
            .asana_get(get_params(ResourceType::Tasks, "proj123"))
            .await
            .unwrap();
        let text = get_response_text(&result);

        assert!(text.contains("Task 1"));
        assert!(text.contains("Task 2"));
    }

    // ========== ServerHandler tests ==========

    #[test]
    fn test_server_info() {
        let client = Client::new("test").unwrap();
        let server = AsanaServer::with_client(client);
        let info = server.get_info();

        assert_eq!(info.server_info.name, "asanaclient-mcp");
        assert!(info.capabilities.tools.is_some());
    }
}
