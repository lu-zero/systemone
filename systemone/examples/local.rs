//! Same questions as `demo`, against a local llama.cpp server.
//!
//! llama.cpp 0.6.0 and later serve `POST /v1/systemone`. 0.9.0 still does.
//! It does not serve `POST /v1/decisions`, so leave the dialect at the default.
//! `GET /v1/models` on that server is the OpenAI list, which `Client::models`
//! does not parse — this example only calls `system_one_raw`.
//!
//! A server with one model loaded ignores `model`. Router mode routes on it,
//! so set `TYPESAFE_DEFAULT_MODEL` to that model's id.
//! `llama-server --version` with a build number before 11361 does not have this route.
//!
//! ```sh
//! llama-server -m path/to/Julia-1-Q8_0.gguf \
//!     --host 127.0.0.1 --port 8080
//! TYPESAFE_BASE_URL=http://127.0.0.1:8080 cargo run --example local -p systemone
//! ```
//!
//! The builder still requires an API key. This example sends `local`. A server
//! started without `--api-key` ignores it.

use std::collections::BTreeMap;

use systemone::{Client, Error, SystemOneRequest, choice, noul, score};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let base_url =
        std::env::var("TYPESAFE_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let client = Client::builder()
        .api_key("local")
        .base_url(base_url)
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
