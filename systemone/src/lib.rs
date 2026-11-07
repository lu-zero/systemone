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
//! HTTP is [`reqwest`] with rustls. Calls need a [`tokio`] runtime. Redirects are off.
//!
//! The protocol types and builders live in `systemone-core` and are re-exported here.

mod client;
mod config;
mod error;
mod resources;

pub use client::Client;
pub use config::{ClientBuilder, Env};
pub use error::Error;
pub use resources::models::Models;
pub use systemone_core::{
    Answer, ChoiceCriteria, ChoiceResponse, Dialect, EntryType, ModelCard, NoulCriteria,
    NoulResponse, Question, Questions, RetryPolicy, ScoreResponse, SystemOneRequest,
    SystemOneResult, Usage, choice, choice_typed, noul, score,
};
