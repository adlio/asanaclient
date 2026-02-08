//! Events API endpoints for incremental sync.

use crate::types::{EventsResponse, EventsSyncReset};
use crate::{Client, Error};

/// Fields to request for events.
const EVENT_FIELDS: &str = "action,change,change.action,change.field,\
    created_at,parent,parent.name,resource,resource.name,type,user,user.name";

/// API for event operations (incremental sync).
pub struct EventsApi<'a> {
    client: &'a Client,
}

impl<'a> EventsApi<'a> {
    /// Create a new events API instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Establish a sync token for a resource by making an initial request.
    ///
    /// The Events API requires an initial call (which returns 412) to obtain
    /// a sync token. This method makes that call and returns the fresh token.
    pub async fn establish(&self, resource_gid: &str) -> Result<String, Error> {
        let url = format!("{}/events", self.client.base_url());
        let query = [("resource", resource_gid), ("opt_fields", EVENT_FIELDS)];

        let response = self.client.http().get(&url).query(&query).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::PRECONDITION_FAILED {
            let body = response.text().await?;
            let reset: EventsSyncReset = serde_json::from_str(&body).map_err(Error::Parse)?;
            Ok(reset.sync)
        } else if status.is_success() {
            // Unexpected success on first call — parse the response and return the token
            let body = response.text().await?;
            let resp: EventsResponse = serde_json::from_str(&body).map_err(Error::Parse)?;
            Ok(resp.sync)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Error::Api {
                message: format!("Events establish failed: HTTP {status} {body}"),
            })
        }
    }

    /// Get events for a resource since the given sync token.
    ///
    /// Automatically drains all pages (following `has_more`) and returns
    /// the aggregated response with the final sync token.
    ///
    /// Returns `Error::SyncTokenExpired { sync }` if the token has expired (412).
    pub async fn get_events(
        &self,
        resource_gid: &str,
        sync: &str,
    ) -> Result<EventsResponse, Error> {
        let mut all_events = Vec::new();
        let mut current_sync = sync.to_string();

        loop {
            let url = format!("{}/events", self.client.base_url());
            let query = [
                ("resource", resource_gid),
                ("sync", current_sync.as_str()),
                ("opt_fields", EVENT_FIELDS),
            ];

            let response = self.client.http().get(&url).query(&query).send().await?;
            let status = response.status();

            if status == reqwest::StatusCode::PRECONDITION_FAILED {
                let body = response.text().await?;
                let reset: EventsSyncReset = serde_json::from_str(&body).map_err(Error::Parse)?;
                return Err(Error::SyncTokenExpired { sync: reset.sync });
            }

            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(Error::Api {
                    message: format!("Events API error: HTTP {status} {body}"),
                });
            }

            let body = response.text().await?;
            let page: EventsResponse = serde_json::from_str(&body).map_err(Error::Parse)?;

            all_events.extend(page.data);
            current_sync = page.sync;

            if !page.has_more {
                break;
            }
        }

        Ok(EventsResponse {
            data: all_events,
            sync: current_sync,
            has_more: false,
        })
    }
}

impl Client {
    /// Access the Events API.
    pub fn events(&self) -> EventsApi<'_> {
        EventsApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::Client;

    fn test_client(server: &MockServer) -> Client {
        Client::new("test-token")
            .unwrap()
            .with_base_url(&server.uri())
    }

    #[tokio::test]
    async fn test_establish_returns_sync_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .respond_with(ResponseTemplate::new(412).set_body_json(serde_json::json!({
                "sync": "fresh_token_abc",
                "errors": [{"message": "Sync token invalid or too old. If you are attempting to keep resources in sync, you must re-fetch the full dataset."}]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let token = client.events().establish("project123").await.unwrap();
        assert_eq!(token, "fresh_token_abc");
    }

    #[tokio::test]
    async fn test_get_events_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "resource": {"gid": "task1", "resource_type": "task", "name": "Task 1"},
                        "action": "changed",
                        "change": {"field": "completed", "action": "changed"}
                    },
                    {
                        "resource": {"gid": "task2", "resource_type": "task", "name": "Task 2"},
                        "action": "added"
                    }
                ],
                "sync": "token_2",
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let resp = client
            .events()
            .get_events("project123", "token_1")
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.sync, "token_2");
        assert!(!resp.has_more);
        assert_eq!(resp.data[0].resource.gid, "task1");
        assert_eq!(resp.data[0].action, "changed");
        assert_eq!(resp.data[1].resource.gid, "task2");
        assert_eq!(resp.data[1].action, "added");
    }

    #[tokio::test]
    async fn test_get_events_has_more() {
        let server = MockServer::start().await;

        // First page
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "resource": {"gid": "task1", "resource_type": "task"},
                        "action": "changed"
                    }
                ],
                "sync": "token_2",
                "has_more": true
            })))
            .mount(&server)
            .await;

        // Second page
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "resource": {"gid": "task2", "resource_type": "task"},
                        "action": "added"
                    }
                ],
                "sync": "token_3",
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let resp = client
            .events()
            .get_events("project123", "token_1")
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.sync, "token_3");
        assert_eq!(resp.data[0].resource.gid, "task1");
        assert_eq!(resp.data[1].resource.gid, "task2");
    }

    #[tokio::test]
    async fn test_get_events_token_expired() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "old_token"))
            .respond_with(ResponseTemplate::new(412).set_body_json(serde_json::json!({
                "sync": "new_fresh_token",
                "errors": [{"message": "Sync token invalid or too old."}]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().get_events("project123", "old_token").await;

        match result {
            Err(crate::Error::SyncTokenExpired { sync }) => {
                assert_eq!(sync, "new_fresh_token");
            }
            other => panic!("Expected SyncTokenExpired, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_get_events_sends_opt_fields() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .and(query_param(
                "opt_fields",
                "action,change,change.action,change.field,created_at,parent,parent.name,resource,resource.name,type,user,user.name",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "sync": "token_2",
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let resp = client
            .events()
            .get_events("project123", "token_1")
            .await
            .unwrap();

        assert!(resp.data.is_empty());
        assert_eq!(resp.sync, "token_2");
    }

    #[tokio::test]
    async fn test_establish_unexpected_success() {
        let server = MockServer::start().await;

        // Unexpected 200 response on first call (should be 412, but API returns 200)
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "resource": {"gid": "task1", "resource_type": "task"},
                        "action": "changed"
                    }
                ],
                "sync": "unexpected_token",
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let token = client.events().establish("project123").await.unwrap();
        assert_eq!(token, "unexpected_token");
    }

    #[tokio::test]
    async fn test_establish_api_error() {
        let server = MockServer::start().await;

        // API returns 500 error
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "errors": [{"message": "Internal server error"}]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().establish("project123").await;

        match result {
            Err(crate::Error::Api { message }) => {
                assert!(
                    message.contains("HTTP 500"),
                    "Expected HTTP 500 in message, got: {}",
                    message
                );
            }
            other => panic!("Expected Api error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_establish_malformed_412_response() {
        let server = MockServer::start().await;

        // 412 response with invalid JSON body
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .respond_with(ResponseTemplate::new(412).set_body_string("not valid json"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().establish("project123").await;

        match result {
            Err(crate::Error::Parse(_)) => {
                // Expected parse error
            }
            other => panic!("Expected Parse error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_events_api_error() {
        let server = MockServer::start().await;

        // API returns 429 rate limit error
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "errors": [{"message": "Rate limit exceeded"}]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().get_events("project123", "token_1").await;

        match result {
            Err(crate::Error::Api { message }) => {
                assert!(
                    message.contains("HTTP 429"),
                    "Expected HTTP 429 in message, got: {}",
                    message
                );
            }
            other => panic!("Expected Api error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_events_malformed_response() {
        let server = MockServer::start().await;

        // 200 response with malformed JSON
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{\"data\": [invalid json"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().get_events("project123", "token_1").await;

        match result {
            Err(crate::Error::Parse(_)) => {
                // Expected parse error
            }
            other => panic!("Expected Parse error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_events_empty_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "sync": "token_2",
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let resp = client
            .events()
            .get_events("project123", "token_1")
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 0);
        assert_eq!(resp.sync, "token_2");
        assert!(!resp.has_more);
    }

    #[tokio::test]
    async fn test_get_events_pagination_error_on_second_page() {
        let server = MockServer::start().await;

        // First page succeeds
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "resource": {"gid": "task1", "resource_type": "task"},
                        "action": "changed"
                    }
                ],
                "sync": "token_2",
                "has_more": true
            })))
            .mount(&server)
            .await;

        // Second page returns error
        Mock::given(method("GET"))
            .and(path("/events"))
            .and(query_param("resource", "project123"))
            .and(query_param("sync", "token_2"))
            .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "errors": [{"message": "Server error"}]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let result = client.events().get_events("project123", "token_1").await;

        match result {
            Err(crate::Error::Api { message }) => {
                assert!(message.contains("HTTP 500"));
            }
            other => panic!("Expected Api error, got: {:?}", other),
        }
    }
}
