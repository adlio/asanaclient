//! Project API endpoints.

use crate::types::requests::{
    AddFollowersData, AddFollowersRequest, AddMembersData, AddMembersRequest, CreateProjectData,
    CreateProjectRequest, InstantiateProjectData, InstantiateProjectRequest, Job,
    RemoveFollowerData, RemoveFollowerRequest, RemoveMembersData, RemoveMembersRequest,
    UpdateProjectData, UpdateProjectRequest,
};
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

    /// Get tasks in a project with full fields for recursive fetching.
    pub(crate) async fn tasks_full(&self, gid: &str) -> Result<Vec<Task>, Error> {
        use crate::api::tasks::RECURSIVE_TASK_FIELDS;
        let path = format!("/projects/{}/tasks", gid);
        let query = [("opt_fields", RECURSIVE_TASK_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    // ========== Write Operations ==========

    /// Create a new project.
    ///
    /// Either `workspace` or `team` must be specified in the data.
    pub async fn create(&self, data: CreateProjectData) -> Result<Project, Error> {
        let path = "/projects".to_string();
        let request = CreateProjectRequest { data };
        self.client.post(&path, &request).await
    }

    /// Instantiate a project from a template.
    ///
    /// Returns a Job that can be polled for completion.
    pub async fn instantiate(
        &self,
        template_gid: &str,
        data: InstantiateProjectData,
    ) -> Result<Job, Error> {
        let path = format!("/project_templates/{}/instantiateProject", template_gid);
        let request = InstantiateProjectRequest { data };
        self.client.post(&path, &request).await
    }

    /// Update a project.
    pub async fn update(&self, gid: &str, data: UpdateProjectData) -> Result<Project, Error> {
        let path = format!("/projects/{}", gid);
        let request = UpdateProjectRequest { data };
        self.client.put(&path, &request).await
    }

    /// Delete a project.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/projects/{}", gid);
        self.client.delete(&path).await
    }

    /// Add members to a project.
    pub async fn add_members(&self, project_gid: &str, member_gids: &[&str]) -> Result<(), Error> {
        let path = format!("/projects/{}/addMembers", project_gid);
        let request = AddMembersRequest {
            data: AddMembersData {
                members: member_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove members from a project.
    pub async fn remove_members(
        &self,
        project_gid: &str,
        member_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/projects/{}/removeMembers", project_gid);
        let request = RemoveMembersRequest {
            data: RemoveMembersData {
                members: member_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add followers to a project.
    pub async fn add_followers(
        &self,
        project_gid: &str,
        follower_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/projects/{}/addFollowers", project_gid);
        let request = AddFollowersRequest {
            data: AddFollowersData {
                followers: follower_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove followers from a project.
    pub async fn remove_followers(
        &self,
        project_gid: &str,
        follower_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/projects/{}/removeFollowers", project_gid);
        let request = RemoveFollowerRequest {
            data: RemoveFollowerData {
                followers: follower_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }
}

impl Client {
    /// Access the projects API.
    pub fn projects(&self) -> ProjectsApi<'_> {
        ProjectsApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_create_project() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "newproj",
                    "name": "New Project",
                    "archived": false
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let project = client
            .projects()
            .create(CreateProjectData {
                name: "New Project".to_string(),
                workspace: Some("ws123".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(project.gid, "newproj");
        assert_eq!(project.name, "New Project");
    }

    #[tokio::test]
    async fn test_instantiate_project() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/project_templates/tmpl123/instantiateProject"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "job123",
                    "resource_type": "job",
                    "status": "in_progress"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let job = client
            .projects()
            .instantiate(
                "tmpl123",
                InstantiateProjectData {
                    name: "From Template".to_string(),
                    team: Some("team456".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(job.gid, "job123");
        assert_eq!(job.status, "in_progress");
    }

    #[tokio::test]
    async fn test_update_project() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/projects/proj123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "proj123",
                    "name": "Updated Project",
                    "archived": true
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let project = client
            .projects()
            .update(
                "proj123",
                UpdateProjectData {
                    name: Some("Updated Project".to_string()),
                    archived: Some(true),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(project.name, "Updated Project");
        assert!(project.archived);
    }

    #[tokio::test]
    async fn test_add_members() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/addMembers"))
            .and(body_json(serde_json::json!({
                "data": {"members": ["user1", "user2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .projects()
            .add_members("proj123", &["user1", "user2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_members() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/removeMembers"))
            .and(body_json(serde_json::json!({
                "data": {"members": ["user1"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .projects()
            .remove_members("proj123", &["user1"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_followers() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/addFollowers"))
            .and(body_json(serde_json::json!({
                "data": {"followers": ["user1"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.projects().add_followers("proj123", &["user1"]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_project() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/projects/proj123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.projects().delete("proj123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_followers() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/removeFollowers"))
            .and(body_json(serde_json::json!({
                "data": {"followers": ["user1", "user2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .projects()
            .remove_followers("proj123", &["user1", "user2"])
            .await;

        assert!(result.is_ok());
    }
}
