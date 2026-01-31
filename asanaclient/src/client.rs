//! HTTP client for the Asana API.

use crate::Error;

const BASE_URL: &str = "https://app.asana.com/api/1.0";
const ENV_VAR: &str = "ASANA_TOKEN";

/// Client for interacting with the Asana API.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    /// Create a new client from the `ASANA_TOKEN` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if `ASANA_TOKEN` is not set or is empty.
    pub fn from_env() -> Result<Self, Error> {
        let token = std::env::var(ENV_VAR).map_err(|_| Error::MissingToken)?;

        if token.is_empty() {
            return Err(Error::MissingToken);
        }

        Self::new(&token)
    }

    /// Create a new client with the given access token.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn new(token: &str) -> Result<Self, Error> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|_| Error::InvalidToken)?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(Error::Http)?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    /// Returns a reference to the underlying HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Returns the base URL for API requests.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = Client::new("test-token").unwrap();
        assert_eq!(client.base_url(), BASE_URL);
    }

    #[test]
    fn test_empty_token_creates_client() {
        // Empty token creates a valid client; the API will reject it at request time
        let result = Client::new("");
        assert!(result.is_ok());
    }
}
