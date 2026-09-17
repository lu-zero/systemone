# systemone

Rust client for the [TypeSafe AI](https://typesafe.ai) `systemone` API — ask named
questions about text or structured state and get back typed, probabilistic answers.
A Rust counterpart to [`@typesafe-ai/sdk`](https://github.com/typesafe-ai/typesafe-sdk-js).

Built on [`isahc`](https://docs.rs/isahc), which is executor-agnostic: this crate makes
no assumption about `tokio`, `smol`, or any other runtime.

## Workspace layout

- `systemone` — the client: HTTP transport, retry, errors, question builders, wire types.
- `systemone-macro` — *stub.* Planned derive macro turning a plain enum into both the
  outbound `choice`/`score` criteria and the typed response value.
- `systemone-facet` — *stub.* Same goal as `systemone-macro`, explored via
  [`facet`](https://github.com/facet-rs/facet) reflection instead of a proc-macro, to
  see which fits better before committing to one.

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

See `systemone/examples/demo.rs` for a runnable version.
