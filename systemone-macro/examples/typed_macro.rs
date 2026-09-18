//! Run with `TYPESAFE_API_KEY=... cargo run --example typed_macro -p systemone-macro`.

use std::collections::BTreeMap;

use serde::Deserialize;
use systemone::{ChoiceResponse, Client, Error, SystemOneRequest, choice_typed};
use systemone_macro::ChoiceCriteria;

#[derive(Debug, Deserialize, ChoiceCriteria)]
#[serde(rename_all = "lowercase")]
enum Sentiment {
    Calm,
    /// Annoyed but not hostile.
    Frustrated,
    Angry,
}

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
