# asanaclient

A Rust SDK and MCP server for the [Asana API](https://developers.asana.com/reference/rest-api-reference).

## Crates

| Crate | Description |
|-------|-------------|
| `asanaclient` | Rust SDK for the Asana API |
| `asanaclient-mcp` | STDIO MCP server exposing Asana operations as tools |

## Features

### asanaclient (SDK)

Read-only operations for Asana's core resources:

- **Tasks** - Search, retrieve, and list tasks with custom field support
- **Projects** - Search, retrieve projects with sections and task counts
- **Portfolios** - List and retrieve portfolios with their items

The SDK handles pagination automatically and provides strongly-typed responses.

### asanaclient-mcp (MCP Server)

A Model Context Protocol server that exposes Asana operations as tools for AI assistants. Designed for use with Claude Desktop, Claude Code, and other MCP-compatible clients.

## Authentication

Authentication uses a Personal Access Token (PAT) via the `ASANA_TOKEN` environment variable.

```bash
export ASANA_TOKEN="your-personal-access-token"
```

To obtain a PAT:
1. Go to [Asana Developer Console](https://app.asana.com/0/developer-console)
2. Create a new Personal Access Token
3. Copy the token and set it as `ASANA_TOKEN`

OAuth support is planned for a future release.

## Installation

### SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
asanaclient = "0.1"
```

### MCP Server

Install the binary:

```bash
cargo install asanaclient-mcp
```

Configure in Claude Desktop's `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "asana": {
      "command": "asanaclient-mcp",
      "env": {
        "ASANA_TOKEN": "your-personal-access-token"
      }
    }
  }
}
```

## Usage

### SDK Example

```rust
use asanaclient::Client;

#[tokio::main]
async fn main() -> Result<(), asanaclient::Error> {
    let client = Client::from_env()?;

    // List workspaces
    let workspaces = client.workspaces().list().await?;

    // Get tasks for a project
    let tasks = client
        .tasks()
        .for_project("project_gid")
        .await?;

    Ok(())
}
```

## Development

```bash
# Run all checks (formatting, linting, build, tests)
make ci

# Run tests
make test

# Generate coverage report
make coverage-html
```

## License

MIT
