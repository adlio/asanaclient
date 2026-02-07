//! Task types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, UserRef};
use super::custom_field::CustomFieldValue;

/// An Asana task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// The unique identifier for the task.
    pub gid: Gid,
    /// The name/title of the task.
    pub name: String,
    /// The resource type (always "task").
    pub resource_type: Option<String>,
    /// Whether the task is completed.
    #[serde(default)]
    pub completed: bool,
    /// When the task was completed, if applicable.
    pub completed_at: Option<String>,
    /// The user who completed the task.
    pub completed_by: Option<UserRef>,
    /// The user assigned to the task.
    pub assignee: Option<UserRef>,
    /// The due date of the task (date only, no time).
    pub due_on: Option<String>,
    /// The due date and time of the task.
    pub due_at: Option<String>,
    /// The start date of the task.
    pub start_on: Option<String>,
    /// The start date and time of the task.
    pub start_at: Option<String>,
    /// Notes/description for the task (plain text).
    pub notes: Option<String>,
    /// Notes/description for the task (HTML).
    pub html_notes: Option<String>,
    /// When the task was created.
    pub created_at: Option<String>,
    /// The user who created the task.
    pub created_by: Option<UserRef>,
    /// When the task was last modified.
    pub modified_at: Option<String>,
    /// Permalink URL for the task.
    pub permalink_url: Option<String>,
    /// The parent task, if this is a subtask.
    pub parent: Option<TaskRef>,
    /// Number of likes on the task.
    #[serde(default)]
    pub num_likes: u32,
    /// Number of subtasks.
    #[serde(default)]
    pub num_subtasks: u32,
    /// Whether the task is liked by the current user.
    #[serde(default)]
    pub liked: bool,
    /// The projects this task belongs to.
    #[serde(default)]
    pub projects: Vec<ResourceRef>,
    /// The workspace this task belongs to.
    pub workspace: Option<ResourceRef>,
    /// The tags on this task.
    #[serde(default)]
    pub tags: Vec<ResourceRef>,
    /// Task memberships (project associations with section info).
    #[serde(default)]
    pub memberships: Vec<TaskMembership>,
    /// The assignee section (column in board view).
    pub assignee_section: Option<ResourceRef>,
    /// Custom field values on this task.
    #[serde(default)]
    pub custom_fields: Vec<CustomFieldValue>,
}

/// A compact reference to a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRef {
    /// The unique identifier for the task.
    pub gid: Gid,
    /// The name of the task.
    pub name: Option<String>,
    /// The resource type (always "task").
    pub resource_type: Option<String>,
}

/// A task's membership in a project (includes section information).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskMembership {
    /// The project this membership is for.
    pub project: ResourceRef,
    /// The section within the project.
    pub section: Option<ResourceRef>,
}

/// A task dependency relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDependency {
    /// The unique identifier for the dependency.
    pub gid: Gid,
    /// The name of the dependent task.
    pub name: Option<String>,
    /// The resource type.
    pub resource_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_task() {
        let json = r#"{
            "gid": "123",
            "name": "My Task",
            "completed": false,
            "due_on": "2024-01-15",
            "notes": "Task description"
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.gid, "123");
        assert_eq!(task.name, "My Task");
        assert!(!task.completed);
        assert_eq!(task.due_on, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_deserialize_task_with_assignee() {
        let json = r#"{
            "gid": "456",
            "name": "Assigned Task",
            "completed": true,
            "assignee": {
                "gid": "789",
                "name": "John Doe"
            }
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert!(task.completed);
        assert!(task.assignee.is_some());
        assert_eq!(task.assignee.unwrap().name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_deserialize_task_ref() {
        let json = r#"{
            "gid": "123",
            "name": "Task Name",
            "resource_type": "task"
        }"#;
        let task_ref: TaskRef = serde_json::from_str(json).unwrap();
        assert_eq!(task_ref.gid, "123");
        assert_eq!(task_ref.name, Some("Task Name".to_string()));
    }

    #[test]
    fn test_deserialize_task_membership() {
        let json = r#"{
            "project": {
                "gid": "111",
                "name": "Project A"
            },
            "section": {
                "gid": "222",
                "name": "In Progress"
            }
        }"#;
        let membership: TaskMembership = serde_json::from_str(json).unwrap();
        assert_eq!(membership.project.gid, "111");
        assert!(membership.section.is_some());
    }
}
