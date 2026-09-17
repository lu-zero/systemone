use std::collections::BTreeMap;
use std::time::Duration;

use crate::client::Client;
use crate::error::Error;
use crate::retry::RetryPolicy;

/// Environment variable names read by [`ClientBuilder::build`].
pub struct Env;

impl Env {
    /// Required API key; used when [`ClientBuilder::api_key`] is omitted.
    pub const API_KEY: &'static str = "TYPESAFE_API_KEY";
    /// API root; defaults to `https://api.typesafe.ai`.
    pub const BASE_URL: &'static str = "TYPESAFE_BASE_URL";
    /// Default model name; defaults to `jev-latest`.
    pub const DEFAULT_MODEL: &'static str = "TYPESAFE_DEFAULT_MODEL";
}

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";
pub(crate) const DEFAULT_MODEL: &str = "jev-latest";
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// A trimmed, nonblank environment value, or `None` for missing or blank values.
fn read_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Builds a [`Client`]. Explicit values take precedence over environment variables, then
/// SDK defaults.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
    timeout: Option<Duration>,
    retry: Option<RetryPolicy>,
    default_headers: BTreeMap<String, String>,
}

impl ClientBuilder {
    /// API key; falls back to `TYPESAFE_API_KEY`.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// API root; falls back to `TYPESAFE_BASE_URL`, then `https://api.typesafe.ai`.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Default model; falls back to `TYPESAFE_DEFAULT_MODEL`, then `jev-latest`.
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    /// Timeout per attempt, without a total retry budget. Default: 10s.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Retry policy override. Default: [`RetryPolicy::default`].
    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Add a header sent with every request.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// Resolve configuration and build the client.
    ///
    /// # Errors
    /// Returns [`Error::MissingApiKey`] if no API key was set and `TYPESAFE_API_KEY` is unset.
    pub fn build(self) -> Result<Client, Error> {
        let api_key = self
            .api_key
            .or_else(|| read_env(Env::API_KEY))
            .ok_or(Error::MissingApiKey(Env::API_KEY))?;
        let base_url = self
            .base_url
            .or_else(|| read_env(Env::BASE_URL))
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        let default_model = self
            .default_model
            .or_else(|| read_env(Env::DEFAULT_MODEL))
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        Client::new(
            api_key,
            base_url,
            default_model,
            self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            self.retry.unwrap_or_default(),
            self.default_headers,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_api_key_without_env_is_an_error() {
        // SAFETY: tests run single-threaded within this process for this module only in
        // practice; still, only touch a variable this test owns.
        unsafe { std::env::remove_var(Env::API_KEY) };
        let err = ClientBuilder::default().build().unwrap_err();
        assert!(matches!(err, Error::MissingApiKey(Env::API_KEY)));
    }

    #[test]
    fn base_url_trims_trailing_slashes() {
        let client = ClientBuilder::default()
            .api_key("k")
            .base_url("https://example.com///")
            .build()
            .unwrap();
        assert_eq!(client.base_url, "https://example.com");
    }
}
