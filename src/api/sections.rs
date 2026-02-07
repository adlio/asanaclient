//! Section API endpoints.

use crate::types::requests::{CreateSectionData, CreateSectionRequest, UpdateSectionRequest};
use crate::types::Section;
use crate::{Client, Error};

/// Fields to request for sections.
pub const SECTION_FIELDS: &str = "gid,name,project,project.name,created_at";

/// API for section operations.
pub struct SectionsApi<'a> {
    client: &'a Client,
}

impl<'a> SectionsApi<'a> {
    /// Create a new sections API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a section by its GID.
    pub async fn get(&self, gid: &str) -> Result<Section, Error> {
        let path = format!("/sections/{}", gid);
        let query = [("opt_fields", SECTION_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// List all sections in a project.
    pub async fn list(&self, project_gid: &str) -> Result<Vec<Section>, Error> {
        let path = format!("/projects/{}/sections", project_gid);
        let query = [("opt_fields", SECTION_FIELDS)];
        self.client.get_all(&path, &query).await
    }

    /// Create a new section in a project.
    pub async fn create(&self, project_gid: &str, name: &str) -> Result<Section, Error> {
        let path = format!("/projects/{}/sections", project_gid);
        let request = CreateSectionRequest {
            data: CreateSectionData {
                name: name.to_string(),
                insert_before: None,
                insert_after: None,
            },
        };
        self.client.post(&path, &request).await
    }

    /// Create a new section with positioning.
    pub async fn create_with_position(
        &self,
        project_gid: &str,
        name: &str,
        insert_before: Option<&str>,
        insert_after: Option<&str>,
    ) -> Result<Section, Error> {
        let path = format!("/projects/{}/sections", project_gid);
        let request = CreateSectionRequest {
            data: CreateSectionData {
                name: name.to_string(),
                insert_before: insert_before.map(String::from),
                insert_after: insert_after.map(String::from),
            },
        };
        self.client.post(&path, &request).await
    }

    /// Update a section's name.
    pub async fn update(&self, gid: &str, name: &str) -> Result<Section, Error> {
        let path = format!("/sections/{}", gid);
        let request = UpdateSectionRequest {
            data: crate::types::requests::UpdateSectionData {
                name: Some(name.to_string()),
            },
        };
        self.client.put(&path, &request).await
    }

    /// Delete a section.
    pub async fn delete(&self, gid: &str) -> Result<(), Error> {
        let path = format!("/sections/{}", gid);
        self.client.delete(&path).await
    }
}

impl Client {
    /// Access the sections API.
    pub fn sections(&self) -> SectionsApi<'_> {
        SectionsApi::new(self)
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
    async fn test_get_section() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sections/sect123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "sect123",
                    "name": "To Do",
                    "project": {"gid": "proj456", "name": "My Project"}
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let section = client.sections().get("sect123").await.unwrap();

        assert_eq!(section.gid, "sect123");
        assert_eq!(section.name, "To Do");
    }

    #[tokio::test]
    async fn test_list_sections() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/projects/proj123/sections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "sect1", "name": "To Do"},
                    {"gid": "sect2", "name": "In Progress"},
                    {"gid": "sect3", "name": "Done"}
                ],
                "next_page": null
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let sections = client.sections().list("proj123").await.unwrap();

        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].name, "To Do");
    }

    #[tokio::test]
    async fn test_create_section() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/sections"))
            .and(body_json(serde_json::json!({
                "data": {"name": "New Section"}
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {"gid": "newsect", "name": "New Section"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let section = client
            .sections()
            .create("proj123", "New Section")
            .await
            .unwrap();

        assert_eq!(section.gid, "newsect");
        assert_eq!(section.name, "New Section");
    }

    #[tokio::test]
    async fn test_update_section() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/sections/sect123"))
            .and(body_json(serde_json::json!({
                "data": {"name": "Renamed Section"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"gid": "sect123", "name": "Renamed Section"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let section = client
            .sections()
            .update("sect123", "Renamed Section")
            .await
            .unwrap();

        assert_eq!(section.name, "Renamed Section");
    }

    #[tokio::test]
    async fn test_delete_section() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/sections/sect123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.sections().delete("sect123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_section_with_position() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/proj123/sections"))
            .and(body_json(serde_json::json!({
                "data": {
                    "name": "New Section",
                    "insert_after": "sect456"
                }
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "data": {"gid": "newsect", "name": "New Section"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let section = client
            .sections()
            .create_with_position("proj123", "New Section", None, Some("sect456"))
            .await
            .unwrap();

        assert_eq!(section.gid, "newsect");
        assert_eq!(section.name, "New Section");
    }
}
