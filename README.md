# systemone

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/systemone.svg)](https://crates.io/crates/systemone)
[![Build Status](https://github.com/lu-zero/systemone/workflows/CI/badge.svg)](https://github.com/lu-zero/systemone/actions?query=workflow:CI)
[![dependency status](https://deps.rs/repo/github/lu-zero/systemone/status.svg)](https://deps.rs/repo/github/lu-zero/systemone)

A Rust client for the [TypeSafe AI](https://typesafe.ai) `systemone` API — ask named
questions about text or structured state and get back typed, probabilistic answers.
A Rust counterpart to [`@typesafe-ai/sdk`](https://github.com/typesafe-ai/typesafe-sdk-js).

HTTP is [`reqwest`](https://docs.rs/reqwest) with rustls. Calls run on a [`tokio`](https://docs.rs/tokio)
runtime. Redirects are off, and each attempt uses the client's timeout.

## Workspace layout

| Crate | Purpose |
|-------|---------|
| [`systemone-core`](systemone-core) | `no_std` + `alloc` protocol: wire types, question builders, `RetryPolicy`, OpenAI dialect. No HTTP. |
| [`systemone`](systemone) | The maki client: depends on the core and re-exports it, plus HTTP transport (`reqwest` on `tokio`), retry loop, env config, models call. |
| [`systemone-macro`](systemone-macro) | `#[derive(ChoiceCriteria)]` — build a `choice` question's labels and descriptions from an enum, at compile time. |
| [`systemone-facet`](systemone-facet) | The same, via [`facet`](https://github.com/facet-rs/facet) reflection at call time instead of a proc-macro. |
| [`systemone-bench`](systemone-bench) | Criterion benchmark comparing the two (not published). |

A `no_std` host that only needs the protocol depends on `systemone-core` directly under
the name the derive macro already uses:

```toml
systemone = { package = "systemone-core", version = "0.2" }
```

`systemone-macro` and `systemone-facet` solve the same problem two different ways —
compile-time codegen vs. runtime reflection — and both are supported long-term rather
than one superseding the other. See their own READMEs for the tradeoffs and
`systemone-bench`'s for the numbers.

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
```

Runnable examples, all asking the same three questions about one ticket:

| Example | Server |
|---------|--------|
| [`systemone/examples/demo.rs`](systemone/examples/demo.rs) | Hosted TypeSafe (`TYPESAFE_API_KEY`). Also lists models. |
| [`systemone/examples/local.rs`](systemone/examples/local.rs) | llama.cpp `POST /v1/systemone`. |
| [`systemone/examples/openai.rs`](systemone/examples/openai.rs) | OpenAI Decisions (`OPENAI_API_KEY`). |

`systemone-macro` and `systemone-facet` each have an example that deserializes a `choice`
answer into an enum.

## Dialects

The default posts that body to `POST /v1/systemone`. `Dialect::OpenAi` changes the path
and the JSON, not the host, the key, or the default model (`jev-latest`). A `noul`
goes out as a `predicate`, and choice and score criteria go out as `choices` and `levels`.
String state is sent as text. An object or array is sent as `input_json` (OpenAI accepts
an object or an array of objects). Null, bool, and number state are sent as the text
`null`, `true`, `false`, or the number's digits.

```rust
use systemone::{Client, Dialect};

let api_key = std::env::var("OPENAI_API_KEY")?;
let client = Client::builder()
    .api_key(api_key)
    .base_url("https://api.openai.com")
    .default_model("gpt-6-luna")
    .dialect(Dialect::OpenAi)
    .build()?;
```

[llama.cpp](https://github.com/ggml-org/llama.cpp) 0.9.0 serves `/v1/systemone` (since 0.6.0)
for a decision GGUF, and does not register `/v1/decisions`. A chat model on that route
returns 501. A server with one model loaded ignores `model`.
[vllm.cpp](https://github.com/mudler/vllm.cpp) is the same: `/v1/systemone` only.
Leave the dialect at its default for both.

```sh
llama-server -m path/to/Julia-1-Q8_0.gguf --host 127.0.0.1 --port 8080
TYPESAFE_BASE_URL=http://127.0.0.1:8080 cargo run --example local -p systemone
```

## Development

```sh
just check   # cargo check --workspace --tests
just test    # cargo test --workspace
just lint    # cargo clippy --workspace --all-targets -- -D warnings
just fmt     # cargo fmt --all
just doc     # cargo doc --workspace --no-deps
just ci      # everything CI runs
```

`systemone-facet` and `systemone-bench` need Rust 1.90+ (the `facet` crate's MSRV);
`systemone-core`, `systemone`, and `systemone-macro` target 1.88, matching
[maki](https://github.com/tontinton/maki)'s.
See [AGENTS.md](AGENTS.md) for full project conventions.

## License

[MIT](LICENSE)
