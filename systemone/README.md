# systemone

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/systemone.svg)](https://crates.io/crates/systemone)
[![Docs.rs](https://docs.rs/systemone/badge.svg)](https://docs.rs/systemone)

Rust client for the [TypeSafe AI](https://typesafe.ai) `systemone` API: send text or
structured state plus named questions, get back typed, probabilistic answers.

Built on [`isahc`](https://docs.rs/isahc), which is executor-agnostic — this crate makes
no assumption about `tokio`, `smol`, or any other runtime. Retry backoff sleeps via
[`async-io`](https://docs.rs/async-io)'s `Timer`, which is likewise runtime-independent.

## Quickstart

Set `TYPESAFE_API_KEY` in your environment, then:

```rust
use std::collections::BTreeMap;
use systemone::{choice, Client, SystemOneRequest};

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

println!("{:?}", result.answers["category"]);
```

`system_one_raw` returns answers as an untyped `BTreeMap<String, Answer>`. Deserialize
into your own type with `system_one::<YourAnswers>(request)` instead for typed fields, or
use the `systemone-macro`/`systemone-facet` crates for typed `choice` questions without
hand-writing both the labels sent to the API and the enum for the response.

## Configuration

`Client::builder()` resolves settings in this order: explicit builder call, then
environment variable, then default.

| Setting | Builder method | Environment variable | Default |
|---------|-----------------|-----------------------|---------|
| API key | `.api_key(..)` | `TYPESAFE_API_KEY` | *(required)* |
| Base URL | `.base_url(..)` | `TYPESAFE_BASE_URL` | `https://api.typesafe.ai` |
| Default model | `.default_model(..)` | `TYPESAFE_DEFAULT_MODEL` | `jev-latest` |
| Timeout per attempt | `.timeout(..)` | — | 10s |
| Retry policy | `.retry(..)` | — | see [`RetryPolicy::default`] |

## Errors and retries

Requests are retried automatically for connection failures and for HTTP statuses in
`RetryPolicy::retryable_statuses` (408/429/5xx by default), honoring a server's
`Retry-After` header up to `RetryPolicy::max_retry_after`. Everything else — an
unretryable status, or retries exhausted — comes back as `Err(Error)`.

## License

MIT, see [`../LICENSE`](../LICENSE).
