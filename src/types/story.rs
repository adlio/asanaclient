//! Story/comment types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, ResourceRef, UserRef};

/// A story on a task (comment, system message, or other activity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Story {
    /// The unique identifier for the story.
    pub gid: Gid,
    /// When the story was created.
    pub created_at: Option<String>,
    /// The user who created the story.
    pub created_by: Option<UserRef>,
    /// The type of story.
    pub resource_subtype: Option<StoryType>,
    /// The text content of the story (plain text).
    pub text: Option<String>,
    /// The HTML content of the story.
    pub html_text: Option<String>,
    /// Whether the story is pinned to the top.
    #[serde(default)]
    pub is_pinned: bool,
    /// Whether the story has been edited.
    #[serde(default)]
    pub is_edited: bool,
    /// The target resource this story is about.
    pub target: Option<ResourceRef>,
    /// Number of likes on this story.
    #[serde(default)]
    pub num_likes: u32,
    /// Whether the current user has liked this story.
    #[serde(default)]
    pub liked: bool,
}

/// The type of story/activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    /// A comment from a user.
    CommentAdded,
    /// Task was assigned.
    AssignedTo,
    /// Task was unassigned.
    Unassigned,
    /// Due date was changed.
    DueDateChanged,
    /// Task was completed.
    MarkedComplete,
    /// Task was marked incomplete.
    MarkedIncomplete,
    /// Task was added to a project.
    AddedToProject,
    /// Task was removed from a project.
    RemovedFromProject,
    /// Task was moved to a section.
    SectionChanged,
    /// Task name was changed.
    NameChanged,
    /// Task notes were changed.
    NotesChanged,
    /// A file was attached.
    AttachmentAdded,
    /// A tag was added.
    TagAdded,
    /// A tag was removed.
    TagRemoved,
    /// A subtask was added.
    SubtaskAdded,
    /// A dependency was added.
    DependencyAdded,
    /// A dependent was added.
    DependentAdded,
    /// Task was duplicated.
    Duplicated,
    /// Enum fallback for unknown types.
    #[serde(other)]
    Unknown,
}

impl Story {
    /// Returns true if this story is a comment.
    pub fn is_comment(&self) -> bool {
        matches!(self.resource_subtype, Some(StoryType::CommentAdded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_story_comment() {
        let json = r#"{
            "gid": "123",
            "created_at": "2024-01-15T10:30:00.000Z",
            "resource_subtype": "comment_added",
            "text": "This is a comment",
            "is_pinned": false
        }"#;
        let story: Story = serde_json::from_str(json).unwrap();
        assert_eq!(story.gid, "123");
        assert!(story.is_comment());
        assert_eq!(story.text, Some("This is a comment".to_string()));
    }

    #[test]
    fn test_deserialize_story_system() {
        let json = r#"{
            "gid": "456",
            "resource_subtype": "marked_complete",
            "text": "John Doe completed this task"
        }"#;
        let story: Story = serde_json::from_str(json).unwrap();
        assert!(!story.is_comment());
        assert_eq!(story.resource_subtype, Some(StoryType::MarkedComplete));
    }

    #[test]
    fn test_deserialize_story_unknown_type() {
        let json = r#"{
            "gid": "789",
            "resource_subtype": "some_new_type",
            "text": "Some activity"
        }"#;
        let story: Story = serde_json::from_str(json).unwrap();
        assert_eq!(story.resource_subtype, Some(StoryType::Unknown));
    }

    #[test]
    fn test_story_type_variants() {
        let assigned: StoryType = serde_json::from_str(r#""assigned_to""#).unwrap();
        assert_eq!(assigned, StoryType::AssignedTo);

        let completed: StoryType = serde_json::from_str(r#""marked_complete""#).unwrap();
        assert_eq!(completed, StoryType::MarkedComplete);
    }
}
