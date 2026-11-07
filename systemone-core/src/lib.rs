#![no_std]

extern crate alloc;

use alloc::string::String;

mod criteria;
pub mod dialect;
mod error;
mod questions;
mod retry;
mod types;

pub use criteria::{ChoiceCriteria, choice_typed};
pub use dialect::Dialect;
pub use error::Error;
pub use questions::{choice, noul, score};
pub use retry::RetryPolicy;
pub use types::{
    Answer, ChoiceResponse, EntryType, NoulCriteria, NoulResponse, Question, Questions,
    ScoreResponse, SystemOneRequest, SystemOneResult, Usage,
};

/// Hosted model metadata, as returned by `GET /v1/models`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelCard {
    pub name: String,
    pub description: String,
    pub release_date: String,
}
