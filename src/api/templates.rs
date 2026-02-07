//! Project template API endpoints.

use crate::types::ProjectTemplate;
use crate::{Client, Error};

/// Fields to request for project templates.
pub const TEMPLATE_FIELDS: &str = "gid,name,description,html_description,owner,owner.name,\
    team,team.name,public,requested_dates,requested_dates.gid,requested_dates.name,\
    requested_dates.description,requested_roles,requested_roles.gid,requested_roles.name,color";

/// API for project template operations.
pub struct TemplatesApi<'a> {
    client: &'a Client,
}

impl<'a> TemplatesApi<'a> {
    /// Create a new templates API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a project template by its GID.
    pub async fn get(&self, gid: &str) -> Result<ProjectTemplate, Error> {
        let path = format!("/project_templates/{}", gid);
        let query = [("opt_fields", TEMPLATE_FIELDS)];
        self.client.get(&path, &query).await
    }

    /// List all project templates in a workspace.
    pub async fn list(&self, workspace_gid: &str) -> Result<Vec<ProjectTemplate>, Error> {
        let path = "/project_templates".to_string();
        let query = [
            ("workspace", workspace_gid),
            ("opt_fields", TEMPLATE_FIELDS),
        ];
        self.client.get_all(&path, &query).await
    }

    /// List project templates for a specific team.
    pub async fn list_for_team(&self, team_gid: &str) -> Result<Vec<ProjectTemplate>, Error> {
        let path = format!("/teams/{}/project_templates", team_gid);
        let query = [("opt_fields", TEMPLATE_FIELDS)];
        self.client.get_all(&path, &query).await
    }
}

impl Client {
    /// Access the templates API.
    pub fn templates(&self) -> TemplatesApi<'_> {
        TemplatesApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_get_template() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/project_templates/tmpl123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gid": "tmpl123",
                    "name": "Sprint Template",
                    "description": "Template for sprint planning",
                    "public": true,
                    "requested_dates": [
                        {"gid": "date1", "name": "Sprint Start"}
                    ],
                    "requested_roles": [
                        {"gid": "role1", "name": "Sprint Lead"}
                    ]
                }
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let template = client.templates().get("tmpl123").await.unwrap();

        assert_eq!(template.gid, "tmpl123");
        assert_eq!(template.name, "Sprint Template");
        assert!(template.public);
        assert_eq!(template.requested_dates.len(), 1);
    }

    #[tokio::test]
    async fn test_list_templates() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/project_templates"))
            .and(query_param("workspace", "ws123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "tmpl1", "name": "Template A"},
                    {"gid": "tmpl2", "name": "Template B"}
                ],
                "next_page": null
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let templates = client.templates().list("ws123").await.unwrap();

        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].name, "Template A");
    }

    #[tokio::test]
    async fn test_list_templates_for_team() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/teams/team123/project_templates"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"gid": "tmpl1", "name": "Team Template"}
                ],
                "next_page": null
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let templates = client.templates().list_for_team("team123").await.unwrap();

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Team Template");
    }
}
