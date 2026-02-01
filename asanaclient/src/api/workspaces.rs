//! Workspace API endpoints.

use crate::types::Workspace;
use crate::{Client, Error};

/// API for workspace operations.
pub struct WorkspacesApi<'a> {
    client: &'a Client,
}

impl<'a> WorkspacesApi<'a> {
    /// Create a new workspaces API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all workspaces accessible to the authenticated user.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let workspaces = client.workspaces().list().await?;
    /// for ws in workspaces {
    ///     println!("{}: {}", ws.gid, ws.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<Workspace>, Error> {
        self.client.get_all("/workspaces", &[]).await
    }

    /// Get a single workspace by its GID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let workspace = client.workspaces().get("12345").await?;
    /// println!("Workspace: {}", workspace.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, gid: &str) -> Result<Workspace, Error> {
        let path = format!("/workspaces/{}", gid);
        self.client.get(&path, &[]).await
    }
}

impl Client {
    /// Access the workspaces API.
    pub fn workspaces(&self) -> WorkspacesApi<'_> {
        WorkspacesApi::new(self)
    }
}
