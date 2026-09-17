use thiserror::Error;

/// Errors returned by [`Client`](crate::Client) methods.
#[derive(Debug, Error)]
pub enum Error {
    /// No API key was set and the environment variable named here was unset or blank.
    #[error("no API key was provided; pass `.api_key(..)` to `Client::builder()` or set `{0}`")]
    MissingApiKey(&'static str),

    /// `SystemOneRequest::questions` was empty.
    #[error("at least one question is required")]
    NoQuestions,

    /// A `score` question was built with fewer than two rubric entries.
    #[error("score criteria must have at least two entries, got {count}")]
    TooFewScoreCriteria { count: usize },

    /// The server returned a non-2xx response after any configured retries.
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
        request_id: Option<String>,
    },

    /// The request could not connect, or the response could not be delivered.
    #[error("connection error: {0}")]
    Connection(#[from] isahc::Error),

    /// The request could not be built.
    #[error(transparent)]
    Http(#[from] isahc::http::Error),

    /// A header name passed via `ClientBuilder::header` was invalid.
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] isahc::http::header::InvalidHeaderName),

    /// A header value passed via `ClientBuilder::header` was invalid.
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] isahc::http::header::InvalidHeaderValue),

    /// The request or response body was not valid JSON for the expected shape.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Reading the response body failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    pub(crate) fn api(status: u16, message: String, request_id: Option<String>) -> Self {
        Self::Api {
            status,
            message,
            request_id,
        }
    }

    /// HTTP status code, when this is an [`Error::Api`].
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Whether this kind of failure is one the client already retries internally
    /// (informational: by the time you observe an error, retries are exhausted).
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api { status, .. } => *status == 408 || *status == 429 || *status >= 500,
            Self::Connection(_) => true,
            _ => false,
        }
    }
}
