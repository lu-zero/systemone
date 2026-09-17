use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Text, a JSON object or array, or `null` for state, instructions, and criteria.
pub type EntryType = serde_json::Value;

/// Questions keyed by the names used to identify their answers.
pub type Questions = BTreeMap<String, Question>;

/// Descriptions of the yes and no outcomes for a [`noul`](crate::noul) question.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NoulCriteria {
    /// Description of the yes outcome.
    #[serde(rename = "true", skip_serializing_if = "Option::is_none")]
    pub is_true: Option<EntryType>,
    /// Description of the no outcome.
    #[serde(rename = "false", skip_serializing_if = "Option::is_none")]
    pub is_false: Option<EntryType>,
}

/// A question identified by its `type` field.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    /// A yes/no question with optional descriptions for either outcome.
    Noul {
        #[serde(skip_serializing_if = "Option::is_none")]
        instructions: Option<EntryType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
    /// A question that assigns a score using an ordered rubric.
    Score {
        instructions: EntryType,
        /// At least two descriptions indexed by score from zero; entries may be `null`.
        criteria: Vec<EntryType>,
    },
    /// A question that selects between named alternatives.
    Choice {
        instructions: EntryType,
        /// Labels mapped to descriptions; a `null` description leaves the label undescribed.
        criteria: BTreeMap<String, EntryType>,
    },
}

/// A yes/no answer.
#[derive(Debug, Clone, Deserialize)]
pub struct NoulResponse {
    /// Probability of a yes answer, from zero to one.
    pub noul: f64,
}

/// A selected label and its probabilities.
///
/// `T` defaults to `String`; deserialize into your own `#[derive(Deserialize)]` enum for a
/// typed choice instead.
#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceResponse<T = String> {
    /// The selected label.
    pub choice: T,
    /// Reported confidence in the selected label.
    pub confidence: f64,
    /// Probabilities keyed by label.
    pub probabilities: BTreeMap<String, f64>,
}

/// An expected score with its rubric and probabilities, both keyed by stringified score index.
#[derive(Debug, Clone, Deserialize)]
pub struct ScoreResponse {
    /// Expected score, which may fall between integer rubric levels.
    pub score: f64,
    /// Reported confidence in the score.
    pub confidence: f64,
    /// Rubric descriptions keyed by stringified score index.
    pub legend: BTreeMap<String, EntryType>,
    /// Probabilities keyed by stringified score index.
    pub probabilities: BTreeMap<String, f64>,
}

/// An answer whose question type wasn't known at compile time.
///
/// Returned by [`Client::system_one_raw`](crate::Client::system_one_raw). Deserialize into
/// your own type via [`Client::system_one`](crate::Client::system_one) for typed answers.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Noul(NoulResponse),
    Choice(ChoiceResponse<String>),
    Score(ScoreResponse),
}

/// Token usage for a request.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Number of input tokens used.
    pub input_tokens: u64,
    /// Number of output tokens used.
    pub output_tokens: u64,
}

/// Answers keyed by question name, with model and usage metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemOneResult<A> {
    /// The model used to answer the request.
    pub model: String,
    /// Answers; a `BTreeMap<String, Answer>` unless deserialized into your own type.
    pub answers: A,
    /// Token usage for the request.
    pub usage: Usage,
}

/// State and named questions for [`Client::system_one`](crate::Client::system_one).
#[derive(Debug, Clone, Serialize)]
pub struct SystemOneRequest {
    /// Text, a JSON object or array, or `null` to evaluate.
    pub state: EntryType,
    /// Nonempty questions keyed by the names used to identify their answers.
    pub questions: Questions,
    /// Model override; `None` uses the client's default model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
