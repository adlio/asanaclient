//! Status update API endpoints.

use crate::types::requests::{
    CreateStatusUpdateData, CreateStatusUpdateRequest, UpdateStatusUpdateData,
    UpdateStatusUpdateRequest,
};
use crate::types::StatusUpdate;
use crate::{Client, Error};

/// Fields to request for status updates.
pub const STATUS_UPDATE_FIELDS: &str = "gid,title,text,html_text,status_type,\
    created_at,created_by,created_by.name,modified_at,parent,parent.name";

/// API for status update operations.
pub struct StatusUpdatesApi<'a> {
    client: &'a Client,
}

impl<'a> StatusUpdatesApi<'a> {
    /// Create a new status updates API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a status update by its GID.
    pub async fn get(&self, gid: &str) -> Result<StatusUpdate, Error> {
        let path = format!("/status_updates/{}", gid);
        let query = [("opt_fields", STATUS_UPDATE_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Create a new status update for a project or portfolio.
    ///
    /// # Arguments
    /// * `parent_gid` - The GID of the project or portfolio
    /// * `status_type` - One of: on_track, at_risk, off_track, on_hold, complete
    /// * `title` - Optional title for the status update
    /// * `text` - Optional text content
    pub async fn create(
        &self,
        parent_gid: &str,
        status_type: &str,
        title: Option<&str>,
        text: Option<&str>,
    ) -> Result<StatusUpdate, Error> {
        let path = "/status_updates".to_string();
        let request = CreateStatusUpdateRequest {
            data: CreateStatusUpdateData {
                parent: parent_gid.to_string(),
                status_type: status_type.to_string(),
                title: title.map(String::from),
                text: text.map(String::from),
                html_text: None,
            },
        };
        self.client.post(&path, &request).await
    }

    /// Create a status update with HTML content.
    pub async fn create_with_html(
        &self,
        parent_gid: &str,
        status_type: &str,
        title: Option<&str>,
        html_text: &str,
    ) -> Result<StatusUpdate, Error> {
        let path = "/status_updates".to_string();
        let request = CreateStatusUpdateRequest {
            data: CreateStatusUpdateData {
                parent: parent_gid.to_string(),
                status_type: status_type.to_string(),
                title: title.map(String::from),
                text: None,
                html_text: Some(html_text.to_string()),
            },
        };
        self.client.post(&path, &request).await
    }

    /// Update an existing status update.
    pub async fn update(
        &self,
        gid: &str,
        data: UpdateStatusUpdateData,
    ) -> Result<StatusUpdate, Error> {
        let path = format!("/status_updates/{}", gid);
        let request = UpdateStatusUpdateRequest { data };
        self.client.put(&path, &request).await
    }

    /// Delete a status update.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/status_updates/{}", gid);
        self.client.delete(&path).await
    }
}

impl Client {
    /// Access the status updates API.
    pub fn status_updates(&self) -> StatusUpdatesApi<'_> {
        StatusUpdatesApi::new(self)
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
    async fn test_get_status_update() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/status_updates/status123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "status123",
                    "title": "On Track",
                    "text": "Everything is going well",
                    "status_type": "on_track"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let status = client.status_updates().get("status123").await.unwrap();

        assert_eq!(status.gid, "status123");
        assert_eq!(status.title, Some("On Track".to_string()));
    }

    #[tokio::test]
    async fn test_create_status_update() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/status_updates"))
            .and(body_json(serde_json::json!({
                "data": {
                    "parent": "proj123",
                    "status_type": "on_track",
                    "title": "Weekly Update",
                    "text": "All tasks completed"
                }
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "newstatus",
                    "title": "Weekly Update",
                    "status_type": "on_track"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let status = client
            .status_updates()
            .create(
                "proj123",
                "on_track",
                Some("Weekly Update"),
                Some("All tasks completed"),
            )
            .await
            .unwrap();

        assert_eq!(status.gid, "newstatus");
        assert_eq!(status.title, Some("Weekly Update".to_string()));
    }

    #[tokio::test]
    async fn test_update_status_update() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/status_updates/status123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "status123",
                    "title": "Updated Title",
                    "status_type": "at_risk"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let status = client
            .status_updates()
            .update(
                "status123",
                UpdateStatusUpdateData {
                    status_type: Some("at_risk".to_string()),
                    title: Some("Updated Title".to_string()),
                    text: None,
                    html_text: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(status.title, Some("Updated Title".to_string()));
    }

    #[tokio::test]
    async fn test_delete_status_update() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/status_updates/status123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.status_updates().delete("status123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_status_update_with_html() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/status_updates"))
            .and(body_json(serde_json::json!({
                "data": {
                    "parent": "proj123",
                    "status_type": "on_track",
                    "title": "HTML Update",
                    "html_text": "<body>Status with <b>HTML</b></body>"
                }
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {
                    "gid": "newstatus",
                    "title": "HTML Update",
                    "status_type": "on_track",
                    "html_text": "<body>Status with <b>HTML</b></body>"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let status = client
            .status_updates()
            .create_with_html(
                "proj123",
                "on_track",
                Some("HTML Update"),
                "<body>Status with <b>HTML</b></body>",
            )
            .await
            .unwrap();

        assert_eq!(status.gid, "newstatus");
        assert_eq!(status.title, Some("HTML Update".to_string()));
    }
}
