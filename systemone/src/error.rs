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

    /// The OpenAI dialect was given an empty model string.
    /// An unset model still uses the client default (`jev-latest`).
    #[error("a model is required to send an OpenAI decision")]
    MissingModel,

    /// An OpenAI decision question has no instructions it can send.
    #[error("question `{question}` needs instructions to send as an OpenAI decision")]
    MissingInstructions { question: String },

    /// An OpenAI choice question has fewer than two options.
    #[error(
        "question `{question}` needs at least two choices to send as an OpenAI decision, got {count}"
    )]
    TooFewChoices { question: String, count: usize },

    /// An OpenAI score question has more than ten levels.
    #[error("question `{question}` has {count} score levels; OpenAI decisions accept at most 10")]
    TooManyScoreLevels { question: String, count: usize },

    /// The OpenAI model refused one question.
    #[error("OpenAI refused question `{question}`")]
    Refused { question: String },

    /// The OpenAI response did not include one of the requested questions.
    #[error("OpenAI response omitted question `{question}`")]
    MissingAnswer { question: String },

    /// An OpenAI answer did not match the question that was sent.
    #[error("OpenAI answer for `{question}` has type `{kind}`")]
    UnexpectedAnswer { question: String, kind: String },

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
