//! Rust SDK for the Asana API.
//!
//! This crate provides a strongly-typed client for interacting with the
//! [Asana REST API](https://developers.asana.com/reference/rest-api-reference).
//!
//! # Authentication
//!
//! The client authenticates using a Personal Access Token (PAT) via the
//! `ASANA_TOKEN` environment variable.
//!
//! # Example
//!
//! ```rust,no_run
//! use asanaclient::Client;
//!
//! # async fn example() -> Result<(), asanaclient::Error> {
//! let client = Client::from_env()?;
//!
//! // List workspaces
//! let workspaces = client.workspaces().list().await?;
//!
//! for workspace in workspaces {
//!     println!("{}: {}", workspace.gid, workspace.name);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod error;

pub use client::Client;
pub use error::Error;

/// Result type alias using the crate's error type.
pub type Result<T> = std::result::Result<T, Error>;
