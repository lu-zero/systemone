//! Same questions as `demo`, translated to OpenAI Decisions (`POST /v1/decisions`).
//!
//! Choice and score criteria become `choices` and `levels`. A `noul` goes out as a
//! `predicate`. Answers come back in the systemone shape.
//!
//! ```sh
//! OPENAI_API_KEY=... cargo run --example openai -p systemone
//! ```

use std::collections::BTreeMap;

use systemone::{Client, Dialect, Error, SystemOneRequest, choice, noul, score};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
        .ok_or(Error::MissingApiKey("OPENAI_API_KEY"))?;
    let client = Client::builder()
        .api_key(api_key)
        .base_url("https://api.openai.com")
        .default_model("gpt-6-luna")
        .dialect(Dialect::OpenAi)
        .build()?;

    let ticket = serde_json::json!({
        "subject": "Charged twice this month",
        "body": "Hi, I see two charges of $49 on my card for August. I only have one \
                 account. Please fix this ASAP, I'm pretty frustrated.",
    });

    let mut questions = BTreeMap::new();
    questions.insert(
        "is_billing".to_string(),
        noul("Is this ticket about billing?".into(), None),
    );
    questions.insert(
        "sentiment".to_string(),
        choice(
            "What is the customer's tone?".into(),
            [("calm", None), ("frustrated", None), ("angry", None)],
        ),
    );
    questions.insert(
        "urgency".to_string(),
        score(
            "How urgent is this ticket?".into(),
            vec![
                "can wait".into(),
                "this week".into(),
                "today".into(),
                "right now".into(),
            ],
        )?,
    );

    let result = client
        .system_one_raw(SystemOneRequest {
            state: ticket,
            questions,
            model: None,
        })
        .await?;

    println!("model: {}", result.model);
    for (name, answer) in &result.answers {
        println!("{name}: {answer:?}");
    }
    println!(
        "tokens: {} in / {} out",
        result.usage.input_tokens, result.usage.output_tokens
    );
    Ok(())
}
