use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use http::Method;
use http::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, RETRY_AFTER, USER_AGENT,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use systemone_core::{Answer, Dialect, RetryPolicy, SystemOneRequest, SystemOneResult, dialect};
use tracing::{debug, info};

use crate::config::ClientBuilder;
use crate::error::Error;
use crate::resources::models::Models;

const REQUEST_ID_HEADER: &str = "x-typesafe-request-id";
const RETRY_COUNT_HEADER: &str = "x-typesafe-retry-count";
const RETRY_AFTER_MS_HEADER: &str = "retry-after-ms";
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Client for the TypeSafe AI API.
#[derive(Debug)]
pub struct Client {
    pub(crate) api_key: String,
    /// API root with trailing slashes removed.
    pub base_url: String,
    /// Model used when a request omits `model`.
    pub default_model: String,
    /// Timeout per attempt, without a total retry budget.
    pub timeout: Duration,
    /// Retry settings applied to every request.
    pub retry: RetryPolicy,
    /// Additional headers sent with each request.
    pub default_headers: BTreeMap<String, String>,
    /// Decision API this client posts to.
    pub dialect: Dialect,
    pub(crate) http: reqwest::Client,
    pub(crate) rng: Mutex<fastrand::Rng>,
}

impl Client {
    /// Start building a client. Explicit values take precedence over environment
    /// variables, then SDK defaults.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Access to the models API resource.
    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    /// Answer named questions with an untyped answer map.
    pub async fn system_one_raw(
        &self,
        request: SystemOneRequest,
    ) -> Result<SystemOneResult<BTreeMap<String, Answer>>, Error> {
        self.system_one(request).await
    }

    /// Answer named questions, deserializing answers into `A`.
    ///
    /// Use `BTreeMap<String, Answer>` (see [`Self::system_one_raw`]) for an untyped answer
    /// map, or your own `#[derive(Deserialize)]` struct for typed answers whose field names
    /// match the question names. [`Dialect::OpenAi`] translates the request and the response,
    /// so `A` stays in the systemone shape either way.
    ///
    /// # Errors
    /// Returns [`Error::NoQuestions`] if `request.questions` is empty.
    pub async fn system_one<A: DeserializeOwned>(
        &self,
        mut request: SystemOneRequest,
    ) -> Result<SystemOneResult<A>, Error> {
        if request.questions.is_empty() {
            return Err(Error::NoQuestions);
        }
        request
            .model
            .get_or_insert_with(|| self.default_model.clone());
        let (path, body) = match self.dialect {
            Dialect::SystemOne => ("/v1/systemone", serde_json::to_vec(&request)?),
            Dialect::OpenAi => (dialect::DECISIONS_PATH, dialect::encode(&request)?),
        };
        let text = self.send(Method::POST, path, Some(&body)).await?;
        let value = match self.dialect {
            Dialect::SystemOne => serde_json::from_str(&text)?,
            Dialect::OpenAi => dialect::decode(&text, &request.questions)?,
        };
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, Error> {
        let body = body.map(serde_json::to_vec).transpose()?;
        let text = self.send(method, path, body.as_deref()).await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn send(&self, method: Method, path: &str, body: Option<&[u8]>) -> Result<String, Error> {
        let url = format!("{}{path}", self.base_url);
        let mut attempt: u32 = 0;
        loop {
            let started = Instant::now();
            let (error, retry_after) = match self.dispatch(&method, &url, body, attempt).await {
                Ok(response) => {
                    let status = response.status();
                    info!(
                        method = %method,
                        path,
                        status = status.as_u16(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "systemone request"
                    );

                    if status.is_success() {
                        return response.text().await.map_err(|err| Error::connection(&err));
                    }

                    let retry_after = retry_after(response.headers());
                    let request_id = response
                        .headers()
                        .get(REQUEST_ID_HEADER)
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_string);
                    let error = Error::Api {
                        status: status.as_u16(),
                        message: response.text().await.unwrap_or_default(),
                        request_id,
                    };

                    if !self.retry.retryable_statuses.contains(&status.as_u16()) {
                        return Err(error);
                    }
                    (error, retry_after)
                }
                Err(error @ Error::Connection(_)) => (error, None),
                Err(error) => return Err(error),
            };

            if attempt >= self.retry.max_retries {
                return Err(error);
            }
            let delay = {
                let mut rng = self.rng.lock().unwrap_or_else(|e| e.into_inner());
                self.retry.delay_for(attempt, retry_after, &mut rng)
            };
            debug!(attempt, delay_ms = delay.as_millis() as u64, %error, "retrying");
            tokio::time::sleep(delay).await;
            attempt += 1;
        }
    }

    async fn dispatch(
        &self,
        method: &Method,
        url: &str,
        body: Option<&[u8]>,
        attempt: u32,
    ) -> Result<reqwest::Response, Error> {
        let mut builder = self
            .http
            .request(method.clone(), url)
            .timeout(self.timeout)
            .header(
                AUTHORIZATION,
                HeaderValue::try_from(format!("Bearer {}", self.api_key))?,
            )
            .header(USER_AGENT, format!("systemone-rs/{SDK_VERSION}"));

        if body.is_some() {
            builder = builder.header(CONTENT_TYPE, "application/json");
        }
        if attempt > 0 {
            builder = builder.header(RETRY_COUNT_HEADER, attempt.to_string());
        }
        for (name, value) in &self.default_headers {
            builder = builder.header(
                HeaderName::try_from(name.as_str())?,
                HeaderValue::try_from(value.as_str())?,
            );
        }
        if let Some(body) = body {
            builder = builder.body(body.to_vec());
        }
        builder.send().await.map_err(|err| Error::connection(&err))
    }
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    if let Some(ms) = headers
        .get(RETRY_AFTER_MS_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
    {
        return Some(Duration::from_millis(ms));
    }
    headers
        .get(RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}
