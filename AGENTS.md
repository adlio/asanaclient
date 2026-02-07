# Agent Instructions for asanaclient

This document provides guidance for AI coding assistants working in this repository.

## Project Overview

This is a Rust SDK providing a strongly-typed client for the Asana API. The goal is to provide a top-notch developer experience for Rust developers integrating with Asana.

**Key URLs:**
- Asana API Reference: https://developers.asana.com/reference/rest-api-reference
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

## Directory Structure

```
asanaclient/
├── Cargo.toml                      # Crate manifest
├── Makefile                        # Build automation
├── src/
│   ├── lib.rs                      # Public API exports
│   ├── client.rs                   # HTTP client with fluent API
│   ├── error.rs                    # Error types
│   ├── api/                        # API facades (tasks.rs, projects.rs, etc.)
│   └── types/                      # Response types (task.rs, project.rs, etc.)
└── docs/
    └── asana_oas.yaml              # Asana OpenAPI spec (reference)
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
5. Add accessor method to `Client` in `client.rs`
6. Re-export from `lib.rs` if it's a commonly-used type
7. Run `make ci`

## Asana API Notes

| Concept | Notes |
|---------|-------|
| **GID** | Global identifier - always a string, never an integer |
| **Pagination** | Offset-based with `next_page.offset` token; `Client::get_all()` handles automatically |
| **Status colors** | Can be color names (`green`) or semantic (`on_track`) - see `StatusColor` enum |
