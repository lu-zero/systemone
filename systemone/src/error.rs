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

    /// The server returned a non-2xx response after any configured retries.
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
        request_id: Option<String>,
    },

    /// The request could not be sent, or the response body could not be read.
    /// The message includes the underlying cause.
    #[error("connection error: {0}")]
    Connection(String),

    /// A header name passed via `ClientBuilder::header` was invalid.
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// A header value passed via `ClientBuilder::header` was invalid.
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// A protocol-level failure from `systemone-core` (question construction or dialect
    /// encode/decode, including its `serde_json` failures).
    #[error(transparent)]
    Core(#[from] systemone_core::Error),

    /// The request or response body was not valid JSON for the expected shape.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// HTTP status code, when this is an [`Error::Api`].
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub(crate) fn connection(err: &impl std::error::Error) -> Self {
        // reqwest's Display is only the top of the source chain.
        let mut message = err.to_string();
        let mut source = err.source();
        while let Some(cause) = source {
            message.push_str(": ");
            message.push_str(&cause.to_string());
            source = cause.source();
        }
        Self::Connection(message)
    }
}
