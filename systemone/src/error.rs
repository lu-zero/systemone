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

    /// [`Dialect::OpenAi`](crate::Dialect::OpenAi) was selected and no model was set.
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
    /// HTTP status code, when this is an [`Error::Api`].
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }
}
