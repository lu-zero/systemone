//! Rust client for the [TypeSafe AI](https://typesafe.ai) `systemone` API.
//!
//! Set `TYPESAFE_API_KEY` in your environment, then:
//!
//! ```no_run
//! use std::collections::BTreeMap;
//! use systemone::{choice, Client, SystemOneRequest};
//!
//! # async fn run() -> Result<(), systemone::Error> {
//! let client = Client::builder().build()?;
//!
//! let mut questions = BTreeMap::new();
//! questions.insert(
//!     "category".to_string(),
//!     choice(
//!         "What is this ticket about?".into(),
//!         [("billing", None), ("technical", None), ("other", None)],
//!     ),
//! );
//!
//! let result = client
//!     .system_one_raw(SystemOneRequest {
//!         state: "I was charged twice. Please fix this ASAP.".into(),
//!         questions,
//!         model: None,
//!     })
//!     .await?;
//! # let _ = result;
//! # Ok(())
//! # }
//! ```
//!
//! Built on [`isahc`], which is executor-agnostic: this crate makes no assumption about
//! `tokio`, `smol`, or any other async runtime.

mod client;
mod config;
mod criteria;
mod error;
mod questions;
mod resources;
mod retry;
mod types;

pub use client::Client;
pub use config::{ClientBuilder, Env};
pub use criteria::{ChoiceCriteria, choice_typed};
pub use error::Error;
pub use questions::{choice, noul, score};
pub use resources::models::ModelCard;
pub use retry::RetryPolicy;
pub use types::{
    Answer, ChoiceResponse, EntryType, NoulCriteria, NoulResponse, Question, Questions,
    ScoreResponse, SystemOneRequest, SystemOneResult, Usage,
};
