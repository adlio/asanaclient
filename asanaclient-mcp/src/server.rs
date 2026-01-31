//! MCP server implementation for Asana.

use asanaclient::Client;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorData as McpError, Implementation, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use rmcp::{schemars, tool, tool_handler, tool_router, ServerHandler};
use serde::Deserialize;

/// MCP server for Asana operations.
#[derive(Debug, Clone)]
pub struct AsanaServer {
    #[allow(dead_code)]
    client: Client,
    tool_router: ToolRouter<AsanaServer>,
}

/// Parameters for listing workspaces.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListWorkspacesParams {}

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

    /// List all workspaces accessible to the authenticated user.
    #[tool(description = "List all workspaces accessible to the authenticated user")]
    async fn asana_list_workspaces(
        &self,
        _params: Parameters<ListWorkspacesParams>,
    ) -> Result<CallToolResult, McpError> {
        // TODO: Implement workspace listing
        Ok(CallToolResult::success(vec![Content::text(
            "Workspaces: (not yet implemented)",
        )]))
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
