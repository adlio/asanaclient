//! Task API endpoints.

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
