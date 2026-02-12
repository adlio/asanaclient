# Agent Instructions for asanaclient

This document provides guidance for AI coding assistants working in this repository.

## Project Overview

This is a Rust SDK providing a strongly-typed client for the Asana API. The goal is to provide a top-notch developer
experience for Rust developers integrating with Asana.

**Key URLs:**

- Asana API Reference: https://developers.asana.com/reference/rest-api-reference
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

## Directory Structure

```
asanaclient/
├── Cargo.toml                          # Crate manifest
├── Makefile                            # Build automation
├── src/
│   ├── lib.rs                          # Public API exports
│   ├── client.rs                       # HTTP client with pagination support
│   ├── error.rs                        # Error types
│   ├── api/                            # API facades
│   │   ├── favorites.rs                # Cross-resource favorites aggregation
│   │   ├── portfolios.rs               # Portfolio operations (recursive expansion)
│   │   ├── projects.rs                 # Project CRUD and search
│   │   ├── sections.rs                 # Section operations
│   │   ├── status_updates.rs           # Status update retrieval
│   │   ├── stories.rs                  # Story/comment operations
│   │   ├── tags.rs                     # Tag CRUD
│   │   ├── tasks.rs                    # Task CRUD, subtasks, dependencies, comments
│   │   ├── templates.rs                # Project template operations
│   │   ├── users.rs                    # User retrieval
│   │   └── workspaces.rs              # Workspace listing
│   └── types/                          # Response/request types
│       ├── common.rs                   # Shared types (Gid, ResourceRef, etc.)
│       ├── custom_field.rs             # Custom field value types
│       ├── portfolio.rs                # Portfolio types
│       ├── project.rs                  # Project types
│       ├── requests.rs                 # Request body types for write operations
│       ├── section.rs                  # Section types
│       ├── status_update.rs            # Status update types
│       ├── story.rs                    # Story/comment types
│       ├── tag.rs                      # Tag types
│       ├── task.rs                     # Task types
│       ├── template.rs                 # Template types
│       ├── user.rs                     # User types
│       └── workspace.rs               # Workspace types
└── docs/
    └── asana_oas.yaml                  # Asana OpenAPI spec (reference)
```

## Code Quality

Before committing, run:

```bash
make ci
```

This checks formatting, linting (clippy warnings are errors), builds, docs, and tests.

**Rules:**

- Never use `anyhow` in library code - use `thiserror`
- All clippy warnings must be fixed
- Write tests for new functionality

## Adding a New API Resource

1. Create type in `types/` (copy pattern from existing types)
2. Export from `types/mod.rs`
3. Create API module in `api/` (copy pattern from existing APIs)
4. Export from `api/mod.rs`
5. Add accessor method to `Client` in the API module file (see bottom of any existing API file)
6. Re-export from `lib.rs` if it's a commonly-used type
7. Run `make ci`

## Releasing

Pushing a version tag triggers the release workflow (`.github/workflows/release.yml`),
which creates a GitHub release and publishes to crates.io.

1. Update `CHANGELOG.md` with the new version's changes under a `## [x.y.z]` heading
2. Bump the version in `Cargo.toml`
3. Commit: `chore: bump version to x.y.z`
4. Tag and push: `git tag vx.y.z && git push origin main --tags`

The workflow will:
- Extract the changelog section for the version and use it as GitHub release notes
- Run `cargo publish` using the `CARGO_REGISTRY_TOKEN` repository secret

## Asana API Notes

| Concept           | Notes                                                                                 |
|-------------------|---------------------------------------------------------------------------------------|
| **GID**           | Global identifier - always a string, never an integer                                 |
| **Pagination**    | Offset-based with `next_page.offset` token; `Client::get_all()` handles automatically |
| **Status colors** | Can be color names (`green`) or semantic (`on_track`) - see `StatusColor` enum        |
