# systemone

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/systemone.svg)](https://crates.io/crates/systemone)
[![Build Status](https://github.com/lu-zero/systemone/workflows/CI/badge.svg)](https://github.com/lu-zero/systemone/actions?query=workflow:CI)
[![dependency status](https://deps.rs/repo/github/lu-zero/systemone/status.svg)](https://deps.rs/repo/github/lu-zero/systemone)

A Rust client for the [TypeSafe AI](https://typesafe.ai) `systemone` API — ask named
questions about text or structured state and get back typed, probabilistic answers.
A Rust counterpart to [`@typesafe-ai/sdk`](https://github.com/typesafe-ai/typesafe-sdk-js).

Built on [`isahc`](https://docs.rs/isahc), which is executor-agnostic: this crate makes
no assumption about `tokio`, `smol`, or any other async runtime.

## Workspace layout

| Crate | Purpose |
|-------|---------|
| [`systemone`](systemone) | The client itself: HTTP transport, retry, errors, question builders, wire types. |
| [`systemone-macro`](systemone-macro) | `#[derive(ChoiceCriteria)]` — build a `choice` question's labels and descriptions from an enum, at compile time. |
| [`systemone-facet`](systemone-facet) | The same, via [`facet`](https://github.com/facet-rs/facet) reflection at call time instead of a proc-macro. |
| [`systemone-bench`](systemone-bench) | Criterion benchmark comparing the two (not published). |

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

See [`systemone/examples/demo.rs`](systemone/examples/demo.rs) for a runnable version,
and `systemone-macro`/`systemone-facet` for typed `choice` answers instead of raw strings.

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
`systemone`/`systemone-macro` target 1.88, matching [maki](https://github.com/tontinton/maki)'s.
See [AGENTS.md](AGENTS.md) for full project conventions.

## License

[MIT](LICENSE)
