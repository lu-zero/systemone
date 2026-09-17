use std::collections::BTreeMap;

use systemone::{Client, SystemOneRequest, choice};

fn main() {
    if let Err(err) = futures_lite::future::block_on(run()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), systemone::Error> {
    let client = Client::builder().build()?;

    let mut questions = BTreeMap::new();
    questions.insert(
        "category".to_string(),
        choice(
            "What is this ticket about?".into(),
            [("billing", None), ("technical", None), ("other", None)],
        ),
    );

    let result = client
        .system_one_raw(SystemOneRequest {
            state: "I was charged twice. Please fix this ASAP.".into(),
            questions,
            model: None,
        })
        .await?;

    println!("model: {}", result.model);
    println!("category: {:?}", result.answers.get("category"));
    Ok(())
}
