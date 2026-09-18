//! Run with `TYPESAFE_API_KEY=... cargo run --example typed -p systemone-facet`.
//!
//! Same scenario as systemone-macro's `typed` example, via reflection instead of a derive.

use std::collections::BTreeMap;

use facet::Facet;
use serde::Deserialize;
use systemone::{ChoiceResponse, Client, Error, SystemOneRequest, choice_typed};

// facet's default label (lowercased variant name) already matches
// #[serde(rename_all = "lowercase")], so no #[facet(rename_all = ...)] is needed here.
#[derive(Debug, Facet, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
enum Sentiment {
    Calm,
    Frustrated,
    Angry,
}
systemone_facet::impl_choice_criteria!(Sentiment);

#[derive(Deserialize)]
struct Answers {
    sentiment: ChoiceResponse<Sentiment>,
}

fn main() {
    if let Err(err) = futures_lite::future::block_on(run()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let client = Client::builder().build()?;

    let mut questions = BTreeMap::new();
    questions.insert(
        "sentiment".to_string(),
        choice_typed::<Sentiment>("What is the customer's tone?".into()),
    );

    let result: systemone::SystemOneResult<Answers> = client
        .system_one(SystemOneRequest {
            state: "This is the third time I've had to email about this. Fix it now.".into(),
            questions,
            model: None,
        })
        .await?;

    let sentiment = &result.answers.sentiment;
    println!(
        "sentiment: {:?} ({:.2} confidence)",
        sentiment.choice, sentiment.confidence
    );
    Ok(())
}
