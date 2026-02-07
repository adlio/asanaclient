//! Story/comment API endpoints.

use crate::types::requests::{UpdateStoryData, UpdateStoryRequest};
use crate::types::Story;
use crate::{Client, Error};

/// Fields to request for stories.
pub const STORY_FIELDS: &str = "gid,created_at,created_by,created_by.name,\
    resource_subtype,text,html_text,is_pinned,is_edited,num_likes,liked";

/// API for story operations.
pub struct StoriesApi<'a> {
    client: &'a Client,
}

impl<'a> StoriesApi<'a> {
    /// Create a new stories API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a story by its GID.
    pub async fn get(&self, gid: &str) -> Result<Story, Error> {
        let path = format!("/stories/{}", gid);
        let query = [("opt_fields", STORY_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// Update a story's text.
    pub async fn update(&self, gid: &str, text: &str) -> Result<Story, Error> {
        let path = format!("/stories/{}", gid);
        let request = UpdateStoryRequest {
            data: UpdateStoryData {
                text: Some(text.to_string()),
                html_text: None,
            },
        };
        self.client.put(&path, &request).await
    }

    /// Update a story with HTML text.
    pub async fn update_html(&self, gid: &str, html_text: &str) -> Result<Story, Error> {
        let path = format!("/stories/{}", gid);
        let request = UpdateStoryRequest {
            data: UpdateStoryData {
                text: None,
                html_text: Some(html_text.to_string()),
            },
        };
        self.client.put(&path, &request).await
    }

    /// Delete a story/comment.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/stories/{}", gid);
        self.client.delete(&path).await
    }
}

impl Client {
    /// Access the stories API.
    pub fn stories(&self) -> StoriesApi<'_> {
        StoriesApi::new(self)
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
    async fn test_get_story() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stories/story123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "story123",
                    "text": "This is a comment",
                    "resource_subtype": "comment_added",
                    "is_pinned": false
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let story = client.stories().get("story123").await.unwrap();

        assert_eq!(story.gid, "story123");
        assert_eq!(story.text, Some("This is a comment".to_string()));
    }

    #[tokio::test]
    async fn test_update_story() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/stories/story123"))
            .and(body_json(serde_json::json!({
                "data": {"text": "Updated comment"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "story123",
                    "text": "Updated comment",
                    "is_edited": true
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let story = client
            .stories()
            .update("story123", "Updated comment")
            .await
            .unwrap();

        assert_eq!(story.text, Some("Updated comment".to_string()));
        assert!(story.is_edited);
    }

    #[tokio::test]
    async fn test_delete_story() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/stories/story123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.stories().delete("story123").await;

        assert!(result.is_ok());
    }
}
