# Agent Instructions for asanaclient

This document provides guidance for AI coding assistants working in this repository.

## Project Overview

This is a Rust monorepo containing two crates:

- **asanaclient** - A Rust SDK for the Asana API (read operations)
- **asanaclient-mcp** - An MCP server exposing Asana tools

## Code Quality Requirements

Before committing any changes, ensure all checks pass:

```bash
make ci
```

This runs:
1. `cargo fmt --all -- --check` - Code formatting
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` - Linting
3. `cargo build --workspace --all-targets --all-features` - Compilation
4. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` - Documentation
5. `cargo nextest run --workspace --all-features` - Tests

### Formatting

- Use `cargo fmt` before committing
- The project follows standard Rust formatting conventions

### Linting

- All Clippy warnings are treated as errors (`-D warnings`)
- Fix all warnings before committing
- Use `#[allow(...)]` sparingly and with justification

### Testing

- Tests use `cargo-nextest` for parallel execution
- Write tests for new functionality
- Run `make test` to verify tests pass

## Architecture Guidelines

### SDK (asanaclient)

- Use `reqwest` for HTTP with async/await
- Use `thiserror` for error types (not `anyhow`)
- Follow Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Pagination should be automatic by default
- All API responses should be strongly typed

### MCP Server (asanaclient-mcp)

- Uses `rmcp` crate for MCP protocol implementation
- Each Asana operation maps to an MCP tool
- Tools should have clear descriptions for LLM understanding
- Tool schemas use JSON Schema via `schemars`

### Error Handling

- Define specific error types with `thiserror`
- Never use `anyhow` in library code
- Provide context in error messages
- Use `Result<T, Error>` consistently

### Dependencies

- Minimize dependencies
- Prefer well-maintained crates with good async support
- Use workspace dependencies in root `Cargo.toml`

## Asana API Reference

When implementing Asana API operations:

- API docs: https://developers.asana.com/reference/rest-api-reference
- Base URL: `https://app.asana.com/api/1.0`
- Authentication: Bearer token via `Authorization` header
- GIDs are string identifiers (not integers)
- Pagination uses `offset` tokens, not page numbers

### Key Concepts

- **Workspace** - Top-level container, users belong to workspaces
- **Project** - Container for tasks, has sections
- **Task** - Work item, can have subtasks, custom fields, multiple parents
- **Portfolio** - Collection of projects
- **Section** - Grouping within a project
- **Custom Fields** - User-defined fields with various types (text, number, enum, date, etc.)

## Directory Structure

```
asanaclient/
├── Cargo.toml              # Workspace root
├── Makefile                # Build automation
├── README.md               # User documentation
├── AGENTS.md               # This file
├── asanaclient/            # SDK crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public API
│       ├── client.rs       # HTTP client
│       ├── error.rs        # Error types
│       └── resources/      # API resource modules
│           ├── mod.rs
│           ├── tasks.rs
│           ├── projects.rs
│           └── portfolios.rs
└── asanaclient-mcp/        # MCP server crate
    ├── Cargo.toml
    └── src/
        ├── main.rs         # Entry point
        └── tools/          # MCP tool implementations
```

## Common Tasks

### Adding a new API resource

1. Create a new module in `asanaclient/src/resources/`
2. Define request/response types with `serde` derives
3. Implement methods on the client
4. Add tests
5. Export from `lib.rs`

### Adding a new MCP tool

1. Add tool definition in `asanaclient-mcp/src/tools/`
2. Implement the tool handler using the SDK
3. Register the tool in the server
4. Add integration tests

### Running tests with coverage

```bash
make coverage-html
```

This generates an HTML report and opens it in your browser.
