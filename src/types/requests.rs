//! Request body types for Asana API write operations.

use serde::Serialize;
use std::collections::HashMap;

// ============================================================================
// Task Operations
// ============================================================================

/// Request body for creating a task.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    /// The task data.
    pub data: CreateTaskData,
}

/// Data for creating a task.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTaskData {
    /// The name of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The workspace to create the task in (required if no projects specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// The projects to add the task to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    /// The assignee of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// The due date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// The start date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Plain text notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// HTML notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<String>,
    /// Whether the task is completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    /// Custom field values (field GID -> value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Request body for updating a task.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTaskRequest {
    /// The task data.
    pub data: UpdateTaskData,
}

/// Data for updating a task.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateTaskData {
    /// The name of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The assignee of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// The due date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// The start date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Plain text notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// HTML notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<String>,
    /// Whether the task is completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    /// Custom field values (field GID -> value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Request body for adding a task to a project.
#[derive(Debug, Clone, Serialize)]
pub struct AddProjectRequest {
    /// The data.
    pub data: AddProjectData,
}

/// Data for adding a task to a project.
#[derive(Debug, Clone, Serialize)]
pub struct AddProjectData {
    /// The project GID to add the task to.
    pub project: String,
    /// The section to add the task to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Insert the task before this task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Insert the task after this task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// Request body for removing a task from a project.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveProjectRequest {
    /// The data.
    pub data: RemoveProjectData,
}

/// Data for removing a task from a project.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveProjectData {
    /// The project GID to remove the task from.
    pub project: String,
}

/// Request body for adding a tag to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddTagRequest {
    /// The data.
    pub data: AddTagData,
}

/// Data for adding a tag to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddTagData {
    /// The tag GID to add.
    pub tag: String,
}

/// Request body for removing a tag from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveTagRequest {
    /// The data.
    pub data: RemoveTagData,
}

/// Data for removing a tag from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveTagData {
    /// The tag GID to remove.
    pub tag: String,
}

/// Request body for setting a task's parent.
#[derive(Debug, Clone, Serialize)]
pub struct SetParentRequest {
    /// The data.
    pub data: SetParentData,
}

/// Data for setting a task's parent.
#[derive(Debug, Clone, Serialize)]
pub struct SetParentData {
    /// The parent task GID, or null to remove the parent.
    pub parent: Option<String>,
    /// Insert before this task in the parent's subtasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Insert after this task in the parent's subtasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// Request body for adding dependencies to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddDependenciesRequest {
    /// The data.
    pub data: AddDependenciesData,
}

/// Data for adding dependencies to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddDependenciesData {
    /// The GIDs of tasks to add as dependencies.
    pub dependencies: Vec<String>,
}

/// Request body for removing dependencies from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveDependenciesRequest {
    /// The data.
    pub data: RemoveDependenciesData,
}

/// Data for removing dependencies from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveDependenciesData {
    /// The GIDs of tasks to remove as dependencies.
    pub dependencies: Vec<String>,
}

/// Request body for adding dependents to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddDependentsRequest {
    /// The data.
    pub data: AddDependentsData,
}

/// Data for adding dependents to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddDependentsData {
    /// The GIDs of tasks to add as dependents.
    pub dependents: Vec<String>,
}

/// Request body for removing dependents from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveDependentsRequest {
    /// The data.
    pub data: RemoveDependentsData,
}

/// Data for removing dependents from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveDependentsData {
    /// The GIDs of tasks to remove as dependents.
    pub dependents: Vec<String>,
}

/// Request body for adding followers to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddFollowersRequest {
    /// The data.
    pub data: AddFollowersData,
}

/// Data for adding followers to a task.
#[derive(Debug, Clone, Serialize)]
pub struct AddFollowersData {
    /// The GIDs of users to add as followers.
    pub followers: Vec<String>,
}

/// Request body for removing a follower from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveFollowerRequest {
    /// The data.
    pub data: RemoveFollowerData,
}

/// Data for removing a follower from a task.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveFollowerData {
    /// The GIDs of users to remove as followers.
    pub followers: Vec<String>,
}

// ============================================================================
// Project Operations
// ============================================================================

/// Request body for creating a project.
#[derive(Debug, Clone, Serialize)]
pub struct CreateProjectRequest {
    /// The project data.
    pub data: CreateProjectData,
}

/// Data for creating a project.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateProjectData {
    /// The name of the project.
    pub name: String,
    /// The workspace to create the project in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// The team to create the project in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    /// The color of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Plain text notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// HTML notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<String>,
    /// The due date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// The start date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// The default view (list, board, calendar, timeline).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_view: Option<String>,
    /// Privacy setting (public_to_workspace or private_to_team).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_setting: Option<String>,
}

/// Request body for updating a project.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateProjectRequest {
    /// The project data.
    pub data: UpdateProjectData,
}

/// Data for updating a project.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateProjectData {
    /// The name of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The color of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Plain text notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// HTML notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<String>,
    /// The due date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// The start date (YYYY-MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Whether the project is archived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// Privacy setting (public_to_workspace or private_to_team).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_setting: Option<String>,
    /// Custom field values (field GID -> value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Request body for instantiating a project from a template.
#[derive(Debug, Clone, Serialize)]
pub struct InstantiateProjectRequest {
    /// The data.
    pub data: InstantiateProjectData,
}

/// Data for instantiating a project from a template.
#[derive(Debug, Clone, Default, Serialize)]
pub struct InstantiateProjectData {
    /// The name of the new project.
    pub name: String,
    /// The team to create the project in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    /// Whether to make the project public.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Date variables for the template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_dates: Option<Vec<DateVariable>>,
    /// Role assignments for the template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_roles: Option<Vec<RoleAssignment>>,
}

/// A date variable for template instantiation.
#[derive(Debug, Clone, Serialize)]
pub struct DateVariable {
    /// The GID of the date variable.
    pub gid: String,
    /// The value (YYYY-MM-DD format).
    pub value: String,
}

/// A role assignment for template instantiation.
#[derive(Debug, Clone, Serialize)]
pub struct RoleAssignment {
    /// The GID of the role.
    pub gid: String,
    /// The GID of the user to assign to the role.
    pub value: String,
}

/// Request body for adding members to a project.
#[derive(Debug, Clone, Serialize)]
pub struct AddMembersRequest {
    /// The data.
    pub data: AddMembersData,
}

/// Data for adding members.
#[derive(Debug, Clone, Serialize)]
pub struct AddMembersData {
    /// The GIDs of users to add as members.
    pub members: Vec<String>,
}

/// Request body for removing members from a project.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveMembersRequest {
    /// The data.
    pub data: RemoveMembersData,
}

/// Data for removing members.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveMembersData {
    /// The GIDs of users to remove as members.
    pub members: Vec<String>,
}

// ============================================================================
// Portfolio Operations
// ============================================================================

/// Request body for creating a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePortfolioRequest {
    /// The portfolio data.
    pub data: CreatePortfolioData,
}

/// Data for creating a portfolio.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreatePortfolioData {
    /// The name of the portfolio.
    pub name: String,
    /// The workspace to create the portfolio in.
    pub workspace: String,
    /// The color of the portfolio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Whether the portfolio is public.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
}

/// Request body for updating a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct UpdatePortfolioRequest {
    /// The portfolio data.
    pub data: UpdatePortfolioData,
}

/// Data for updating a portfolio.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdatePortfolioData {
    /// The name of the portfolio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The color of the portfolio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Whether the portfolio is public.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
}

/// Request body for adding an item to a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct AddItemRequest {
    /// The data.
    pub data: AddItemData,
}

/// Data for adding an item to a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct AddItemData {
    /// The GID of the project to add.
    pub item: String,
    /// Insert before this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Insert after this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// Request body for removing an item from a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveItemRequest {
    /// The data.
    pub data: RemoveItemData,
}

/// Data for removing an item from a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct RemoveItemData {
    /// The GID of the item to remove.
    pub item: String,
}

// ============================================================================
// Section Operations
// ============================================================================

/// Request body for creating a section.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSectionRequest {
    /// The section data.
    pub data: CreateSectionData,
}

/// Data for creating a section.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSectionData {
    /// The name of the section.
    pub name: String,
    /// Insert before this section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Insert after this section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// Request body for updating a section.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSectionRequest {
    /// The section data.
    pub data: UpdateSectionData,
}

/// Data for updating a section.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateSectionData {
    /// The name of the section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// Status Update Operations
// ============================================================================

/// Request body for creating a status update.
#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusUpdateRequest {
    /// The status update data.
    pub data: CreateStatusUpdateData,
}

/// Data for creating a status update.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStatusUpdateData {
    /// The GID of the project or portfolio.
    pub parent: String,
    /// The status type (on_track, at_risk, off_track, on_hold, complete).
    pub status_type: String,
    /// The title of the status update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Plain text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
}

/// Request body for updating a status update.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusUpdateRequest {
    /// The status update data.
    pub data: UpdateStatusUpdateData,
}

/// Data for updating a status update.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStatusUpdateData {
    /// The status type (on_track, at_risk, off_track, on_hold, complete).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_type: Option<String>,
    /// The title of the status update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Plain text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
}

// ============================================================================
// Story/Comment Operations
// ============================================================================

/// Request body for creating a comment on a task.
#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentRequest {
    /// The comment data.
    pub data: CreateCommentData,
}

/// Data for creating a comment.
#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentData {
    /// Plain text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
}

/// Request body for updating a story/comment.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateStoryRequest {
    /// The story data.
    pub data: UpdateStoryData,
}

/// Data for updating a story/comment.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateStoryData {
    /// Plain text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
}

// ============================================================================
// Tag Operations
// ============================================================================

/// Request body for creating a tag.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTagRequest {
    /// The tag data.
    pub data: CreateTagData,
}

/// Data for creating a tag.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTagData {
    /// The name of the tag.
    pub name: String,
    /// The workspace to create the tag in.
    pub workspace: String,
    /// The color of the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Notes about the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for updating a tag.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTagRequest {
    /// The tag data.
    pub data: UpdateTagData,
}

/// Data for updating a tag.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateTagData {
    /// The name of the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The color of the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Notes about the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ============================================================================
// Job Response (for async operations like template instantiation)
// ============================================================================

/// A job returned by asynchronous operations.
#[derive(Debug, Clone, serde::Deserialize, Serialize)]
pub struct Job {
    /// The unique identifier for the job.
    pub gid: String,
    /// The resource type (always "job").
    pub resource_type: String,
    /// The status of the job (queued, in_progress, succeeded, failed).
    pub status: String,
    /// The new project created (if applicable and completed).
    pub new_project: Option<JobProject>,
}

/// A compact project reference in a job.
#[derive(Debug, Clone, serde::Deserialize, Serialize)]
pub struct JobProject {
    /// The GID of the project.
    pub gid: String,
    /// The name of the project.
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_request_serialization() {
        let request = CreateTaskRequest {
            data: CreateTaskData {
                name: Some("Test Task".to_string()),
                workspace: Some("ws123".to_string()),
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Test Task\""));
        assert!(json.contains("\"workspace\":\"ws123\""));
    }

    #[test]
    fn test_update_task_request_skips_none() {
        let request = UpdateTaskRequest {
            data: UpdateTaskData {
                name: Some("Updated".to_string()),
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Updated\""));
        assert!(!json.contains("assignee"));
        assert!(!json.contains("due_on"));
    }

    #[test]
    fn test_add_project_request_serialization() {
        let request = AddProjectRequest {
            data: AddProjectData {
                project: "proj123".to_string(),
                section: Some("sect456".to_string()),
                insert_before: None,
                insert_after: None,
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"project\":\"proj123\""));
        assert!(json.contains("\"section\":\"sect456\""));
    }

    #[test]
    fn test_instantiate_project_request() {
        let request = InstantiateProjectRequest {
            data: InstantiateProjectData {
                name: "New Project".to_string(),
                team: Some("team123".to_string()),
                requested_dates: Some(vec![DateVariable {
                    gid: "date1".to_string(),
                    value: "2024-12-31".to_string(),
                }]),
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"New Project\""));
        assert!(json.contains("\"requested_dates\""));
    }
}
