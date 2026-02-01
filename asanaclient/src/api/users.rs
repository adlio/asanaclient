//! User API endpoints.

use crate::types::{FavoriteItem, User};
use crate::{Client, Error};

/// API for user operations.
pub struct UsersApi<'a> {
    client: &'a Client,
}

impl<'a> UsersApi<'a> {
    /// Create a new users API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the currently authenticated user.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let me = client.users().me().await?;
    /// println!("Logged in as: {}", me.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn me(&self) -> Result<User, Error> {
        self.client.get("/users/me", &[]).await
    }

    /// Get the favorites for the current user in a workspace.
    ///
    /// Returns all favorited items (projects, portfolios, etc.) for the
    /// authenticated user in the specified workspace.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let favorites = client.users().favorites("12345").await?;
    /// for item in favorites {
    ///     println!("{}: {} ({})", item.gid, item.name.unwrap_or_default(), item.resource_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn favorites(&self, workspace_gid: &str) -> Result<Vec<FavoriteItem>, Error> {
        let path = "/users/me/favorites";
        let query = [
            ("workspace", workspace_gid),
            ("resource_type", "project,portfolio"),
        ];
        self.client.get_all(path, &query).await
    }

    /// Get only favorited projects for the current user in a workspace.
    pub async fn favorite_projects(&self, workspace_gid: &str) -> Result<Vec<FavoriteItem>, Error> {
        let path = "/users/me/favorites";
        let query = [("workspace", workspace_gid), ("resource_type", "project")];
        self.client.get_all(path, &query).await
    }

    /// Get only favorited portfolios for the current user in a workspace.
    pub async fn favorite_portfolios(
        &self,
        workspace_gid: &str,
    ) -> Result<Vec<FavoriteItem>, Error> {
        let path = "/users/me/favorites";
        let query = [("workspace", workspace_gid), ("resource_type", "portfolio")];
        self.client.get_all(path, &query).await
    }
}

impl Client {
    /// Access the users API.
    pub fn users(&self) -> UsersApi<'_> {
        UsersApi::new(self)
    }
}
