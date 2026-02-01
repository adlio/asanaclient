//! Task API endpoints.

use crate::api::portfolios::PortfolioItemExpanded;
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
        // Default portfolio_depth to 3 (same as get_portfolio)
        let portfolio_depth = portfolio_depth.unwrap_or(3);

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
