use alloc::string::String;

use thiserror::Error;

/// Protocol errors: question construction and dialect encode/decode failures.
#[derive(Debug, Error)]
pub enum Error {
    /// A `score` question was built with fewer than two rubric entries.
    #[error("score criteria must have at least two entries, got {count}")]
    TooFewScoreCriteria { count: usize },

    /// The OpenAI dialect was given an empty model string.
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

    /// The request or response body was not valid JSON for the expected shape.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}
