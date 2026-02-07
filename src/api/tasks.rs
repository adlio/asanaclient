//! Task API endpoints.

use crate::api::portfolios::PortfolioItemExpanded;
use crate::types::requests::{
    AddDependenciesData, AddDependenciesRequest, AddDependentsData, AddDependentsRequest,
    AddFollowersData, AddFollowersRequest, AddProjectData, AddProjectRequest, AddTagData,
    AddTagRequest, CreateCommentData, CreateCommentRequest, CreateTaskData, CreateTaskRequest,
    RemoveDependenciesData, RemoveDependenciesRequest, RemoveDependentsData,
    RemoveDependentsRequest, RemoveFollowerData, RemoveFollowerRequest, RemoveProjectData,
    RemoveProjectRequest, RemoveTagData, RemoveTagRequest, SetParentData, SetParentRequest,
    UpdateTaskData, UpdateTaskRequest,
};
use crate::types::{Story, Task, TaskDependency, TaskRef};
use crate::{Client, Error};

/// Fields to request for a basic task fetch.
pub const TASK_FIELDS: &str = "gid,name,resource_type,completed,completed_at,\
    assignee,assignee.name,due_on,due_at,start_on,notes,created_at,modified_at,\
    permalink_url,parent,num_likes,num_subtasks,liked,projects,projects.name,\
    workspace,tags,memberships,memberships.project,memberships.project.name,\
    memberships.section,memberships.section.name";

/// Fields to request for a full task fetch.
pub const TASK_FULL_FIELDS: &str = "gid,name,resource_type,completed,completed_at,\
    completed_by,completed_by.name,assignee,assignee.name,assignee.email,\
    due_on,due_at,start_on,start_at,notes,html_notes,created_at,created_by,\
    created_by.name,modified_at,permalink_url,parent,parent.name,num_likes,\
    num_subtasks,liked,projects,projects.name,workspace,workspace.name,\
    tags,tags.name,memberships,memberships.project,memberships.project.name,\
    memberships.section,memberships.section.name,assignee_section,\
    assignee_section.name";

/// Fields to request for subtasks.
pub const SUBTASK_FIELDS: &str = "gid,name,completed,assignee,assignee.name,\
    due_on,num_subtasks";

/// Fields to request for recursive task fetching (includes project refs).
pub const RECURSIVE_TASK_FIELDS: &str = "gid,name,resource_type,completed,completed_at,\
    assignee,assignee.name,due_on,due_at,start_on,notes,created_at,modified_at,\
    permalink_url,parent,parent.name,num_likes,num_subtasks,liked,\
    projects,projects.name,workspace,tags,memberships,memberships.project,\
    memberships.project.name,memberships.section,memberships.section.name";

/// Fields to request for stories/comments.
pub const STORY_FIELDS: &str = "gid,created_at,created_by,created_by.name,\
    resource_subtype,text,html_text,is_pinned,is_edited,num_likes,liked";

/// API for task operations.
pub struct TasksApi<'a> {
    client: &'a Client,
}

impl<'a> TasksApi<'a> {
    /// Create a new tasks API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a task by its GID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let task = client.tasks().get("12345").await?;
    /// println!("Task: {} (completed: {})", task.name, task.completed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, gid: &str) -> Result<Task, Error> {
        let path = format!("/tasks/{}", gid);
        let query = [("opt_fields", TASK_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Get a task with full details.
    pub async fn get_full(&self, gid: &str) -> Result<Task, Error> {
        let path = format!("/tasks/{}", gid);
        let query = [("opt_fields", TASK_FULL_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Get subtasks of a task.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    /// let subtasks = client.tasks().subtasks("12345").await?;
    /// for subtask in subtasks {
    ///     println!("  - {} (completed: {})", subtask.name, subtask.completed);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subtasks(&self, gid: &str) -> Result<Vec<Task>, Error> {
        let path = format!("/tasks/{}/subtasks", gid);
        let query = [("opt_fields", SUBTASK_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    /// Get tasks that this task depends on (blockers).
    pub async fn dependencies(&self, gid: &str) -> Result<Vec<TaskDependency>, Error> {
        let path = format!("/tasks/{}/dependencies", gid);
        let query = [("opt_fields", "gid,name,resource_type")];
        self.client.get_all(&path, &query).await
    }

    /// Get tasks that depend on this task (blocked by this task).
    pub async fn dependents(&self, gid: &str) -> Result<Vec<TaskDependency>, Error> {
        let path = format!("/tasks/{}/dependents", gid);
        let query = [("opt_fields", "gid,name,resource_type")];
        self.client.get_all(&path, &query).await
    }

    /// Get all stories (comments and activity) for a task.
    pub async fn stories(&self, gid: &str) -> Result<Vec<Story>, Error> {
        let path = format!("/tasks/{}/stories", gid);
        let query = [("opt_fields", STORY_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    /// Get only comments for a task (filters out system messages).
    pub async fn comments(&self, gid: &str) -> Result<Vec<Story>, Error> {
        let stories = self.stories(gid).await?;
        Ok(stories.into_iter().filter(|s| s.is_comment()).collect())
    }

    /// Get subtasks with full fields for recursive fetching.
    pub(crate) async fn subtasks_full(&self, gid: &str) -> Result<Vec<Task>, Error> {
        let path = format!("/tasks/{}/subtasks", gid);
        let query = [("opt_fields", RECURSIVE_TASK_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    // ========== Write Operations ==========

    /// Create a new task.
    ///
    /// Either `workspace` or `projects` must be specified in the data.
    pub async fn create(&self, data: CreateTaskData) -> Result<Task, Error> {
        let path = "/tasks".to_string();
        let request = CreateTaskRequest { data };
        self.client.post(&path, &request).await
    }

    /// Create a subtask under a parent task.
    pub async fn create_subtask(
        &self,
        parent_gid: &str,
        data: CreateTaskData,
    ) -> Result<Task, Error> {
        let path = format!("/tasks/{}/subtasks", parent_gid);
        let request = CreateTaskRequest { data };
        self.client.post(&path, &request).await
    }

    /// Update a task.
    pub async fn update(&self, gid: &str, data: UpdateTaskData) -> Result<Task, Error> {
        let path = format!("/tasks/{}", gid);
        let request = UpdateTaskRequest { data };
        self.client.put(&path, &request).await
    }

    /// Delete a task.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/tasks/{}", gid);
        self.client.delete(&path).await
    }

    /// Add a task to a project.
    pub async fn add_project(
        &self,
        task_gid: &str,
        project_gid: &str,
        section_gid: Option<&str>,
    ) -> Result<(), Error> {
        let path = format!("/tasks/{}/addProject", task_gid);
        let request = AddProjectRequest {
            data: AddProjectData {
                project: project_gid.to_string(),
                section: section_gid.map(String::from),
                insert_before: None,
                insert_after: None,
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove a task from a project.
    pub async fn remove_project(&self, task_gid: &str, project_gid: &str) -> Result<(), Error> {
        let path = format!("/tasks/{}/removeProject", task_gid);
        let request = RemoveProjectRequest {
            data: RemoveProjectData {
                project: project_gid.to_string(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add a tag to a task.
    pub async fn add_tag(&self, task_gid: &str, tag_gid: &str) -> Result<(), Error> {
        let path = format!("/tasks/{}/addTag", task_gid);
        let request = AddTagRequest {
            data: AddTagData {
                tag: tag_gid.to_string(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove a tag from a task.
    pub async fn remove_tag(&self, task_gid: &str, tag_gid: &str) -> Result<(), Error> {
        let path = format!("/tasks/{}/removeTag", task_gid);
        let request = RemoveTagRequest {
            data: RemoveTagData {
                tag: tag_gid.to_string(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Set the parent of a task.
    ///
    /// Pass `None` for `parent_gid` to remove the parent (make it a top-level task).
    pub async fn set_parent(
        &self,
        task_gid: &str,
        parent_gid: Option<&str>,
    ) -> Result<Task, Error> {
        let path = format!("/tasks/{}/setParent", task_gid);
        let request = SetParentRequest {
            data: SetParentData {
                parent: parent_gid.map(String::from),
                insert_before: None,
                insert_after: None,
            },
        };
        self.client.post(&path, &request).await
    }

    /// Add dependencies to a task (tasks that must be completed before this one).
    pub async fn add_dependencies(
        &self,
        task_gid: &str,
        dependency_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/tasks/{}/addDependencies", task_gid);
        let request = AddDependenciesRequest {
            data: AddDependenciesData {
                dependencies: dependency_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove dependencies from a task.
    pub async fn remove_dependencies(
        &self,
        task_gid: &str,
        dependency_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/tasks/{}/removeDependencies", task_gid);
        let request = RemoveDependenciesRequest {
            data: RemoveDependenciesData {
                dependencies: dependency_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add dependents to a task (tasks that depend on this one).
    pub async fn add_dependents(
        &self,
        task_gid: &str,
        dependent_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/tasks/{}/addDependents", task_gid);
        let request = AddDependentsRequest {
            data: AddDependentsData {
                dependents: dependent_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove dependents from a task.
    pub async fn remove_dependents(
        &self,
        task_gid: &str,
        dependent_gids: &[&str],
    ) -> Result<(), Error> {
        let path = format!("/tasks/{}/removeDependents", task_gid);
        let request = RemoveDependentsRequest {
            data: RemoveDependentsData {
                dependents: dependent_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Add followers to a task.
    pub async fn add_followers(&self, task_gid: &str, follower_gids: &[&str]) -> Result<(), Error> {
        let path = format!("/tasks/{}/addFollowers", task_gid);
        let request = AddFollowersRequest {
            data: AddFollowersData {
                followers: follower_gids.iter().map(|s| s.to_string()).collect(),
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Remove a follower from a task.
    pub async fn remove_follower(&self, task_gid: &str, follower_gid: &str) -> Result<(), Error> {
        let path = format!("/tasks/{}/removeFollowers", task_gid);
        let request = RemoveFollowerRequest {
            data: RemoveFollowerData {
                followers: vec![follower_gid.to_string()],
            },
        };
        self.client.post_empty(&path, &request).await
    }

    /// Create a comment on a task.
    pub async fn create_comment(&self, task_gid: &str, text: &str) -> Result<Story, Error> {
        let path = format!("/tasks/{}/stories", task_gid);
        let request = CreateCommentRequest {
            data: CreateCommentData {
                text: Some(text.to_string()),
                html_text: None,
            },
        };
        self.client.post(&path, &request).await
    }

    /// Create a comment with HTML content on a task.
    pub async fn create_comment_html(
        &self,
        task_gid: &str,
        html_text: &str,
    ) -> Result<Story, Error> {
        let path = format!("/tasks/{}/stories", task_gid);
        let request = CreateCommentRequest {
            data: CreateCommentData {
                text: None,
                html_text: Some(html_text.to_string()),
            },
        };
        self.client.post(&path, &request).await
    }
}

/// A task with its related data expanded.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskWithContext {
    /// The task details.
    #[serde(flatten)]
    pub task: Task,
    /// Subtasks of this task.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subtasks: Vec<TaskRef>,
    /// Tasks this task depends on (blockers).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TaskDependency>,
    /// Tasks that depend on this task.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependents: Vec<TaskDependency>,
    /// Comments on this task.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Story>,
}

impl Client {
    /// Access the tasks API.
    pub fn tasks(&self) -> TasksApi<'_> {
        TasksApi::new(self)
    }

    /// Get all tasks recursively from a project or portfolio.
    ///
    /// This function auto-detects whether the GID refers to a project or portfolio:
    /// - If a project: returns all tasks in that project
    /// - If a portfolio: returns all tasks from all projects in the portfolio
    ///   (including nested portfolios up to `portfolio_depth`)
    ///
    /// The `subtask_depth` parameter controls subtask expansion:
    /// - `None` - Unlimited depth (fetch all nested subtasks)
    /// - `Some(0)` - No subtasks (top-level tasks only)
    /// - `Some(n)` - Fetch n levels of subtasks
    ///
    /// The `portfolio_depth` parameter controls how deep to search for projects
    /// (only applies when GID is a portfolio):
    /// - `None` - Unlimited depth
    /// - `Some(0)` - Only direct child projects (not nested portfolios)
    /// - `Some(n)` - Search n levels of nested portfolios (default: 3)
    ///
    /// Returns a flat `Vec<Task>`. Each task includes ALL projects it belongs to
    /// (not just the ones in the queried hierarchy). Use the `parent` field to
    /// reconstruct task hierarchy if needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use asanaclient::Client;
    /// # async fn example() -> Result<(), asanaclient::Error> {
    /// let client = Client::from_env()?;
    ///
    /// // Get all tasks from a project (no subtasks)
    /// let tasks = client.get_tasks_recursive("project_gid", Some(0), None).await?;
    ///
    /// // Get all tasks from a portfolio with unlimited subtask depth
    /// let tasks = client.get_tasks_recursive("portfolio_gid", None, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tasks_recursive(
        &self,
        gid: &str,
        subtask_depth: Option<i32>,
        portfolio_depth: Option<i32>,
    ) -> Result<Vec<Task>, Error> {
        // Default portfolio_depth to 0 (no expansion)
        let portfolio_depth = portfolio_depth.unwrap_or(0);

        // Try to detect resource type by attempting to fetch as project first
        match self.projects().get(gid).await {
            Ok(_) => {
                // It's a project, get tasks from it
                self.get_tasks_from_project(gid, subtask_depth).await
            }
            Err(Error::NotFound(_)) => {
                // Not a project, try as portfolio
                self.get_tasks_from_portfolio(gid, subtask_depth, portfolio_depth)
                    .await
            }
            Err(e) => Err(e),
        }
    }

    /// Get tasks from a single project with optional subtask expansion.
    async fn get_tasks_from_project(
        &self,
        project_gid: &str,
        subtask_depth: Option<i32>,
    ) -> Result<Vec<Task>, Error> {
        let tasks = self.projects().tasks_full(project_gid).await?;
        self.expand_subtasks_flat(tasks, subtask_depth, 0).await
    }

    /// Get tasks from all projects in a portfolio (recursively).
    async fn get_tasks_from_portfolio(
        &self,
        portfolio_gid: &str,
        subtask_depth: Option<i32>,
        portfolio_depth: i32,
    ) -> Result<Vec<Task>, Error> {
        // Convert portfolio_depth to Option<usize> for get_portfolio_recursive
        let depth = if portfolio_depth < 0 {
            None
        } else {
            Some(portfolio_depth as usize)
        };
        let portfolio = self.get_portfolio_recursive(portfolio_gid, depth).await?;
        let project_gids = Self::collect_project_gids_from_portfolio(&portfolio);

        let mut all_tasks = Vec::new();
        for project_gid in project_gids {
            match self
                .get_tasks_from_project(&project_gid, subtask_depth)
                .await
            {
                Ok(tasks) => all_tasks.extend(tasks),
                Err(Error::NotFound(_)) => continue, // Project may have been deleted
                Err(e) => return Err(e),
            }
        }
        Ok(all_tasks)
    }

    /// Collect all project GIDs from a portfolio structure.
    fn collect_project_gids_from_portfolio(
        portfolio: &crate::api::portfolios::PortfolioWithItems,
    ) -> Vec<String> {
        let mut gids = Vec::new();
        for item in &portfolio.items {
            match item {
                PortfolioItemExpanded::Project(p) => gids.push(p.gid.clone()),
                PortfolioItemExpanded::Portfolio(nested) => {
                    gids.extend(Self::collect_project_gids_from_portfolio(nested));
                }
            }
        }
        gids
    }

    /// Expand subtasks into a flat list.
    fn expand_subtasks_flat<'a>(
        &'a self,
        tasks: Vec<Task>,
        subtask_depth: Option<i32>,
        current_depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Task>, Error>> + Send + 'a>>
    {
        Box::pin(async move {
            // Convert depth parameter
            let max_depth = match subtask_depth {
                Some(d) if d < 0 => None, // Unlimited
                Some(d) => Some(d as usize),
                None => None, // Default to unlimited
            };

            // Check if we should fetch subtasks at this depth
            let should_fetch_subtasks = match max_depth {
                None => true,
                Some(max) => current_depth < max,
            };

            let mut all_tasks = Vec::new();

            for task in tasks {
                let has_subtasks = task.num_subtasks > 0;
                all_tasks.push(task.clone());

                if should_fetch_subtasks && has_subtasks {
                    let subtasks = self.tasks().subtasks_full(&task.gid).await?;
                    let expanded = self
                        .expand_subtasks_flat(subtasks, subtask_depth, current_depth + 1)
                        .await?;
                    all_tasks.extend(expanded);
                }
            }

            Ok(all_tasks)
        })
    }

    /// Get a task with full context including subtasks, dependencies, and comments.
    pub async fn get_task_with_context(
        &self,
        gid: &str,
        include_subtasks: bool,
        include_dependencies: bool,
        include_comments: bool,
    ) -> Result<TaskWithContext, Error> {
        let task = self.tasks().get_full(gid).await?;

        let subtasks = if include_subtasks {
            self.tasks()
                .subtasks(gid)
                .await?
                .into_iter()
                .map(|t| TaskRef {
                    gid: t.gid,
                    name: Some(t.name),
                    resource_type: t.resource_type,
                })
                .collect()
        } else {
            Vec::new()
        };

        let (dependencies, dependents) = if include_dependencies {
            let deps = self.tasks().dependencies(gid).await?;
            let depts = self.tasks().dependents(gid).await?;
            (deps, depts)
        } else {
            (Vec::new(), Vec::new())
        };

        let comments = if include_comments {
            self.tasks().comments(gid).await?
        } else {
            Vec::new()
        };

        Ok(TaskWithContext {
            task,
            subtasks,
            dependencies,
            dependents,
            comments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::requests::{CreateTaskData, UpdateTaskData};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_create_task() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "newtask",
                    "name": "New Task",
                    "completed": false
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let task = client
            .tasks()
            .create(CreateTaskData {
                name: Some("New Task".to_string()),
                workspace: Some("ws123".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(task.gid, "newtask");
        assert_eq!(task.name, "New Task");
    }

    #[tokio::test]
    async fn test_create_subtask() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/parent123/subtasks"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "subtask1",
                    "name": "Subtask",
                    "completed": false
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let task = client
            .tasks()
            .create_subtask(
                "parent123",
                CreateTaskData {
                    name: Some("Subtask".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(task.gid, "subtask1");
        assert_eq!(task.name, "Subtask");
    }

    #[tokio::test]
    async fn test_update_task() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/tasks/task123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "task123",
                    "name": "Updated Task",
                    "completed": true
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let task = client
            .tasks()
            .update(
                "task123",
                UpdateTaskData {
                    name: Some("Updated Task".to_string()),
                    completed: Some(true),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(task.name, "Updated Task");
        assert!(task.completed);
    }

    #[tokio::test]
    async fn test_add_project() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/addProject"))
            .and(body_json(serde_json::json!({
                "data": {
                    "project": "proj456",
                    "section": "sect789"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .tasks()
            .add_project("task123", "proj456", Some("sect789"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_project() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/removeProject"))
            .and(body_json(serde_json::json!({
                "data": {"project": "proj456"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tasks().remove_project("task123", "proj456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_tag() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/addTag"))
            .and(body_json(serde_json::json!({
                "data": {"tag": "tag456"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tasks().add_tag("task123", "tag456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_parent() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/setParent"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "task123",
                    "name": "Task",
                    "completed": false
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let task = client
            .tasks()
            .set_parent("task123", Some("parent456"))
            .await
            .unwrap();

        assert_eq!(task.gid, "task123");
    }

    #[tokio::test]
    async fn test_add_dependencies() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/addDependencies"))
            .and(body_json(serde_json::json!({
                "data": {"dependencies": ["dep1", "dep2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .tasks()
            .add_dependencies("task123", &["dep1", "dep2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_followers() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/addFollowers"))
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
            .tasks()
            .add_followers("task123", &["user1", "user2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_comment() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/stories"))
            .and(body_json(serde_json::json!({
                "data": {"text": "This is a comment"}
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "story123",
                    "text": "This is a comment",
                    "resource_subtype": "comment_added"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let story = client
            .tasks()
            .create_comment("task123", "This is a comment")
            .await
            .unwrap();

        assert_eq!(story.gid, "story123");
        assert_eq!(story.text, Some("This is a comment".to_string()));
    }

    #[tokio::test]
    async fn test_delete_task() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/tasks/task123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tasks().delete("task123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_tag() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/removeTag"))
            .and(body_json(serde_json::json!({
                "data": {"tag": "tag456"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tasks().remove_tag("task123", "tag456").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_dependencies() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/removeDependencies"))
            .and(body_json(serde_json::json!({
                "data": {"dependencies": ["dep1", "dep2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .tasks()
            .remove_dependencies("task123", &["dep1", "dep2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_dependents() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/addDependents"))
            .and(body_json(serde_json::json!({
                "data": {"dependents": ["dep1", "dep2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .tasks()
            .add_dependents("task123", &["dep1", "dep2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_dependents() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/removeDependents"))
            .and(body_json(serde_json::json!({
                "data": {"dependents": ["dep1", "dep2"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client
            .tasks()
            .remove_dependents("task123", &["dep1", "dep2"])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_follower() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/removeFollowers"))
            .and(body_json(serde_json::json!({
                "data": {"followers": ["user1"]}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tasks().remove_follower("task123", "user1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_comment_html() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tasks/task123/stories"))
            .and(body_json(serde_json::json!({
                "data": {"html_text": "<body>HTML comment</body>"}
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "story123",
                    "html_text": "<body>HTML comment</body>",
                    "resource_subtype": "comment_added"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let story = client
            .tasks()
            .create_comment_html("task123", "<body>HTML comment</body>")
            .await
            .unwrap();

        assert_eq!(story.gid, "story123");
        assert_eq!(
            story.html_text,
            Some("<body>HTML comment</body>".to_string())
        );
    }
}
