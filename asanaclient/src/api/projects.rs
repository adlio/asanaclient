//! Project API endpoints.

use crate::types::{Project, StatusUpdate, Task};
use crate::{Client, Error};

/// Fields to request for a basic project fetch.
pub const PROJECT_FIELDS: &str = "gid,name,color,archived,public,owner,team,workspace,\
    current_status_update,current_status_update.status_type,current_status_update.title,\
    notes,created_at,modified_at,due_date,due_on,start_on,permalink_url,icon";

/// Fields to request for a full project fetch including status details.
pub const PROJECT_FULL_FIELDS: &str = "gid,name,color,archived,public,owner,owner.name,\
    team,team.name,workspace,workspace.name,current_status_update,\
    current_status_update.gid,current_status_update.status_type,\
    current_status_update.title,current_status_update.text,\
    current_status_update.created_at,current_status_update.created_by,\
    notes,html_notes,created_at,modified_at,due_date,due_on,start_on,\
    default_view,is_template,permalink_url,icon";

/// API for project operations.
pub struct ProjectsApi<'a> {
    client: &'a Client,
}

impl<'a> ProjectsApi<'a> {
    /// Create a new projects API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a project by its GID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let project = client.projects().get("12345").await?;
    /// println!("Project: {}", project.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, gid: &str) -> Result<Project, Error> {
        let path = format!("/projects/{}", gid);
        let query = [("opt_fields", PROJECT_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Get a project with full details including expanded status updates.
    pub async fn get_full(&self, gid: &str) -> Result<Project, Error> {
        let path = format!("/projects/{}", gid);
        let query = [("opt_fields", PROJECT_FULL_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Get status updates for a project.
    ///
    /// Returns the history of status updates for the project.
    pub async fn status_updates(&self, gid: &str) -> Result<Vec<StatusUpdate>, Error> {
        let path = format!("/projects/{}/status_updates", gid);
        let query = [(
            "opt_fields",
            "gid,title,text,html_text,status_type,created_at,created_by,created_by.name",
        )];
        self.client.get_all(&path, &query).await
    }

    /// Get the latest status update for a project.
    ///
    /// Returns `None` if the project has no status updates.
    pub async fn latest_status_update(&self, gid: &str) -> Result<Option<StatusUpdate>, Error> {
        let updates = self.status_updates(gid).await?;
        Ok(updates.into_iter().next())
    }

    /// Get tasks in a project.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let tasks = client.projects().tasks("12345").await?;
    /// for task in tasks {
    ///     println!("  - {} (completed: {})", task.name, task.completed);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn tasks(&self, gid: &str) -> Result<Vec<Task>, Error> {
        let path = format!("/projects/{}/tasks", gid);
        let query = [(
            "opt_fields",
            "gid,name,completed,assignee,assignee.name,due_on,num_subtasks",
        )];
        self.client.get_all(&path, &query).await
    }
}

impl Client {
    /// Access the projects API.
    pub fn projects(&self) -> ProjectsApi<'_> {
        ProjectsApi::new(self)
    }
}
