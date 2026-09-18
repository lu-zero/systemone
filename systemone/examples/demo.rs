//! Run with `TYPESAFE_API_KEY=... cargo run --example demo -p systemone`.

use std::collections::BTreeMap;

use systemone::{Client, Error, SystemOneRequest, choice, noul, score};

fn main() {
    if let Err(err) = futures_lite::future::block_on(run()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let client = Client::builder().build()?;

    let models: Vec<_> = client
        .models()
        .list()
        .await?
        .into_iter()
        .map(|m| m.name)
        .collect();
    println!("available models: {}", models.join(", "));

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
        .await;

    let result = match result {
        Ok(result) => result,
        Err(Error::Api {
            status,
            message,
            request_id,
        }) => {
            eprintln!("API error {status} (request {request_id:?}): {message}");
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    for (name, answer) in &result.answers {
        println!("{name}: {answer:?}");
    }
    println!(
        "tokens: {} in / {} out",
        result.usage.input_tokens, result.usage.output_tokens
    );
    Ok(())
}
