use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use isahc::config::Configurable;
use isahc::http::Method;
use isahc::http::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderName, HeaderValue, RETRY_AFTER, USER_AGENT,
};
use isahc::{AsyncReadResponseExt, HttpClient, Request};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{debug, info};

use crate::config::ClientBuilder;
use crate::error::Error;
use crate::resources::models::Models;
use crate::retry::RetryPolicy;
use crate::types::{Answer, SystemOneRequest, SystemOneResult};

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
    pub(crate) http: HttpClient,
}

impl Client {
    /// Start building a client. Explicit values take precedence over environment
    /// variables, then SDK defaults.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub(crate) fn new(
        api_key: String,
        base_url: String,
        default_model: String,
        timeout: Duration,
        retry: RetryPolicy,
        default_headers: BTreeMap<String, String>,
    ) -> Result<Self, Error> {
        let http = HttpClient::new()?;
        Ok(Self {
            api_key,
            base_url,
            default_model,
            timeout,
            retry,
            default_headers,
            http,
        })
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
    /// match the question names.
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
        self.request(Method::POST, "/v1/systemone", Some(&request))
            .await
    }

    pub(crate) async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, Error> {
        let url = format!("{}{path}", self.base_url);
        let body_bytes = body.map(serde_json::to_vec).transpose()?;

        let mut attempt: u32 = 0;
        loop {
            let request = self.build_request(&method, &url, body_bytes.as_deref(), attempt)?;
            let started = Instant::now();

            match self.http.send_async(request).await {
                Ok(mut response) => {
                    let status = response.status();
                    info!(
                        method = %method,
                        path,
                        status = status.as_u16(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "systemone request"
                    );

                    if status.is_success() {
                        let text = response.text().await?;
                        return Ok(serde_json::from_str(&text)?);
                    }

                    let text = response.text().await.unwrap_or_default();
                    let request_id = response
                        .headers()
                        .get(REQUEST_ID_HEADER)
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_string);

                    let retries_left = self.retry.max_retries.saturating_sub(attempt);
                    let retryable = self.retry.retryable_statuses.contains(&status.as_u16());
                    if !retryable || retries_left == 0 {
                        return Err(Error::api(status.as_u16(), text, request_id));
                    }

                    let delay = self
                        .retry
                        .delay_for(attempt, retry_after(response.headers()));
                    debug!(attempt, delay_ms = delay.as_millis() as u64, %status, "retrying");
                    async_io::Timer::after(delay).await;
                    attempt += 1;
                }
                Err(err) => {
                    let retries_left = self.retry.max_retries.saturating_sub(attempt);
                    if retries_left == 0 {
                        return Err(err.into());
                    }
                    let delay = self.retry.delay_for(attempt, None);
                    debug!(attempt, delay_ms = delay.as_millis() as u64, error = %err, "retrying");
                    async_io::Timer::after(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    fn build_request(
        &self,
        method: &Method,
        url: &str,
        body: Option<&[u8]>,
        attempt: u32,
    ) -> Result<Request<Vec<u8>>, Error> {
        let mut builder = Request::builder()
            .method(method.clone())
            .uri(url)
            .timeout(self.timeout)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(USER_AGENT, format!("systemone-rs/{SDK_VERSION}"));

        if body.is_some() {
            builder = builder.header(CONTENT_TYPE, "application/json");
        }
        if attempt > 0 {
            builder = builder.header(
                HeaderName::from_static(RETRY_COUNT_HEADER),
                attempt.to_string(),
            );
        }
        for (name, value) in &self.default_headers {
            builder = builder.header(
                HeaderName::try_from(name.as_str())?,
                HeaderValue::try_from(value.as_str())?,
            );
        }

        Ok(builder.body(body.map(<[u8]>::to_vec).unwrap_or_default())?)
    }
}

fn retry_after(headers: &isahc::http::HeaderMap) -> Option<Duration> {
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
