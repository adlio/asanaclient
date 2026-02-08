//! Types for the Asana Events API.

use serde::Deserialize;

use super::common::{ResourceRef, UserRef};

/// An event from the Asana Events API.
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// The user who triggered the event.
    pub user: Option<UserRef>,
    /// The resource that was affected.
    pub resource: ResourceRef,
    /// The resource type (e.g. "task", "story").
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    /// The action that occurred: changed, added, removed, deleted, undeleted.
    pub action: String,
    /// The parent resource (e.g. the project for a task event).
    pub parent: Option<ResourceRef>,
    /// When the event was created.
    pub created_at: Option<String>,
    /// Details about what changed.
    pub change: Option<EventChange>,
}

/// Details about what changed in an event.
#[derive(Debug, Clone, Deserialize)]
pub struct EventChange {
    /// The field that changed.
    pub field: Option<String>,
    /// The type of change: changed, added, removed.
    pub action: Option<String>,
    /// The new value of the field.
    pub new_value: Option<serde_json::Value>,
    /// The value that was added.
    pub added_value: Option<serde_json::Value>,
    /// The value that was removed.
    pub removed_value: Option<serde_json::Value>,
}

/// Response from the Events API (200 success).
#[derive(Debug, Clone, Deserialize)]
pub struct EventsResponse {
    /// The events returned.
    pub data: Vec<Event>,
    /// The sync token for the next request.
    pub sync: String,
    /// Whether there are more events to fetch.
    pub has_more: bool,
}

/// Response from the Events API (412 token expired).
/// Contains a fresh sync token and an error message.
#[derive(Debug, Clone, Deserialize)]
pub struct EventsSyncReset {
    /// The fresh sync token to use after a full re-sync.
    pub sync: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_event() {
        let json = r#"{
            "user": {"gid": "123", "name": "Test User"},
            "resource": {"gid": "456", "resource_type": "task", "name": "My Task"},
            "type": "task",
            "action": "changed",
            "parent": {"gid": "789", "resource_type": "project", "name": "My Project"},
            "created_at": "2024-01-01T00:00:00.000Z",
            "change": {
                "field": "completed",
                "action": "changed",
                "new_value": true
            }
        }"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.action, "changed");
        assert_eq!(event.resource.gid, "456");
        assert_eq!(event.resource_type, Some("task".to_string()));
        assert_eq!(event.user.unwrap().gid, "123");
        assert_eq!(event.parent.unwrap().gid, "789");
        let change = event.change.unwrap();
        assert_eq!(change.field, Some("completed".to_string()));
        assert_eq!(change.action, Some("changed".to_string()));
    }

    #[test]
    fn test_deserialize_events_response() {
        let json = r#"{
            "data": [
                {
                    "resource": {"gid": "1", "resource_type": "task"},
                    "action": "changed"
                }
            ],
            "sync": "token123",
            "has_more": false
        }"#;
        let resp: EventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.sync, "token123");
        assert!(!resp.has_more);
    }

    #[test]
    fn test_deserialize_events_sync_reset() {
        let json = r#"{
            "sync": "fresh_token",
            "errors": [{"message": "Sync token invalid or too old"}]
        }"#;
        let reset: EventsSyncReset = serde_json::from_str(json).unwrap();
        assert_eq!(reset.sync, "fresh_token");
    }

    #[test]
    fn test_deserialize_event_minimal() {
        let json = r#"{
            "resource": {"gid": "1"},
            "action": "added"
        }"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.action, "added");
        assert!(event.user.is_none());
        assert!(event.parent.is_none());
        assert!(event.change.is_none());
        assert!(event.resource_type.is_none());
    }
}
