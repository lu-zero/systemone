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

`ClientBuilder::dialect(Dialect::OpenAi)` posts the same questions to OpenAI's
`POST /v1/decisions` and maps the answers back. The default dialect is
`POST /v1/systemone`. That is the route
[llama.cpp](https://github.com/ggml-org/llama.cpp) 0.9.0 (since 0.6.0) and
[vllm.cpp](https://github.com/mudler/vllm.cpp) serve. Neither registers
`/v1/decisions`.

| Example | What it calls |
|---------|----------------|
| `cargo run --example demo -p systemone` | Hosted TypeSafe. Needs `TYPESAFE_API_KEY`. |
| `cargo run --example local -p systemone` | `POST /v1/systemone` on `TYPESAFE_BASE_URL`, or `http://127.0.0.1:8080`. |
| `cargo run --example openai -p systemone` | OpenAI Decisions. Needs `OPENAI_API_KEY`. |

`local` does not call `GET /v1/models`. llama.cpp's list is the OpenAI shape, which
[`ModelCard`] does not parse. Load a decision GGUF. A chat model returns 501. One
loaded model ignores the request's `model` field. Julia-1 Q8_0 is at
`/mnt/models/ggml-org-Julia-1-GGUF/Julia-1-Q8_0.gguf`.

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
