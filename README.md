# asanaclient

[![Crates.io](https://img.shields.io/crates/v/asanaclient.svg)](https://crates.io/crates/asanaclient)
[![docs.rs](https://img.shields.io/docsrs/asanaclient)](https://docs.rs/asanaclient)
[![CI](https://github.com/adlio/asanaclient/actions/workflows/ci.yml/badge.svg)](https://github.com/adlio/asanaclient/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/adlio/asanaclient/branch/main/graph/badge.svg)](https://codecov.io/gh/adlio/asanaclient)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust SDK for the [Asana API](https://developers.asana.com/reference/rest-api-reference).

## Features

- **Tasks** - CRUD operations, subtasks, dependencies, followers, comments, tags, and recursive fetching across projects/portfolios
- **Projects** - List, search, retrieve projects with sections and task listings
- **Portfolios** - List, retrieve portfolios with recursive item expansion
- **Sections** - List and manage project sections
- **Tags** - List, create, and manage tags
- **Stories** - Retrieve and create comments/activity on tasks
- **Status Updates** - Retrieve status updates for projects and portfolios
- **Templates** - List and retrieve project templates
- **Users** - Get current user and list workspace members
- **Workspaces** - List and retrieve workspaces

Automatic pagination, strongly-typed responses, and custom field support are built in.

## Authentication

Authentication uses a Personal Access Token (PAT) via the `ASANA_TOKEN` environment variable.

```bash
export ASANA_TOKEN="your-personal-access-token"
```

To obtain a PAT:
1. Go to [Asana Developer Console](https://app.asana.com/0/developer-console)
2. Create a new Personal Access Token
3. Copy the token and set it as `ASANA_TOKEN`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
asanaclient = "0.1"
```

## Usage

```rust
use asanaclient::Client;

#[tokio::main]
async fn main() -> Result<(), asanaclient::Error> {
    let client = Client::from_env()?;

    // List workspaces
    let workspaces = client.workspaces().list().await?;

    // Get a task by GID
    let task = client.tasks().get("12345").await?;
    println!("{}: {}", task.gid, task.name);

    // Get subtasks
    let subtasks = client.tasks().subtasks("12345").await?;

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
