//! Tag API endpoints.

use crate::types::requests::{CreateTagData, CreateTagRequest, UpdateTagData, UpdateTagRequest};
use crate::types::Tag;
use crate::{Client, Error};

/// Fields to request for tags.
pub const TAG_FIELDS: &str =
    "gid,name,color,notes,workspace,workspace.name,created_at,permalink_url";

/// API for tag operations.
pub struct TagsApi<'a> {
    client: &'a Client,
}

impl<'a> TagsApi<'a> {
    /// Create a new tags API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a tag by its GID.
    pub async fn get(&self, gid: &str) -> Result<Tag, Error> {
        let path = format!("/tags/{}", gid);
        let query = [("opt_fields", TAG_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// List all tags in a workspace.
    pub async fn list(&self, workspace_gid: &str) -> Result<Vec<Tag>, Error> {
        let path = "/tags".to_string();
        let query = [("workspace", workspace_gid), ("opt_fields", TAG_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    /// Create a new tag in a workspace.
    pub async fn create(&self, workspace_gid: &str, name: &str) -> Result<Tag, Error> {
        let path = "/tags".to_string();
        let request = CreateTagRequest {
            data: CreateTagData {
                name: name.to_string(),
                workspace: workspace_gid.to_string(),
                color: None,
                notes: None,
            },
        };
        self.client.post(&path, &request).await
    }

    /// Create a new tag with options.
    pub async fn create_with_options(
        &self,
        workspace_gid: &str,
        name: &str,
        color: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Tag, Error> {
        let path = "/tags".to_string();
        let request = CreateTagRequest {
            data: CreateTagData {
                name: name.to_string(),
                workspace: workspace_gid.to_string(),
                color: color.map(String::from),
                notes: notes.map(String::from),
            },
        };
        self.client.post(&path, &request).await
    }

    /// Update a tag.
    pub async fn update(&self, gid: &str, data: UpdateTagData) -> Result<Tag, Error> {
        let path = format!("/tags/{}", gid);
        let request = UpdateTagRequest { data };
        self.client.put(&path, &request).await
    }

    /// Delete a tag.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/tags/{}", gid);
        self.client.delete(&path).await
    }
}

impl Client {
    /// Access the tags API.
    pub fn tags(&self) -> TagsApi<'_> {
        TagsApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_get_tag() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tags/tag123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "tag123",
                    "name": "Priority",
                    "color": "red"
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let tag = client.tags().get("tag123").await.unwrap();

        assert_eq!(tag.gid, "tag123");
        assert_eq!(tag.name, "Priority");
        assert_eq!(tag.color, Some("red".to_string()));
    }

    #[tokio::test]
    async fn test_list_tags() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tags"))
            .and(query_param("workspace", "ws123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "tag1", "name": "Bug"},
                    {"gid": "tag2", "name": "Feature"}
                ],
                "next_page": null
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let tags = client.tags().list("ws123").await.unwrap();

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "Bug");
    }

    #[tokio::test]
    async fn test_create_tag() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tags"))
            .and(body_json(serde_json::json!({
                "data": {
                    "name": "New Tag",
                    "workspace": "ws123"
                }
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {"gid": "newtag", "name": "New Tag"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let tag = client.tags().create("ws123", "New Tag").await.unwrap();

        assert_eq!(tag.gid, "newtag");
        assert_eq!(tag.name, "New Tag");
    }

    #[tokio::test]
    async fn test_update_tag() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/tags/tag123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"gid": "tag123", "name": "Renamed", "color": "blue"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let tag = client
            .tags()
            .update(
                "tag123",
                UpdateTagData {
                    name: Some("Renamed".to_string()),
                    color: Some("blue".to_string()),
                    notes: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(tag.name, "Renamed");
        assert_eq!(tag.color, Some("blue".to_string()));
    }

    #[tokio::test]
    async fn test_delete_tag() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/tags/tag123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.tags().delete("tag123").await;

        assert!(result.is_ok());
    }
}
