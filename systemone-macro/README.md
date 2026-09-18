# systemone-macro

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/systemone-macro.svg)](https://crates.io/crates/systemone-macro)
[![Docs.rs](https://docs.rs/systemone-macro/badge.svg)](https://docs.rs/systemone-macro)

`#[derive(ChoiceCriteria)]` for [`systemone`](../systemone): builds a `choice` question's
outbound labels and descriptions from an enum's unit variants and doc comments, at compile
time, instead of listing them by hand.

```rust
use systemone::ChoiceCriteria;
use systemone_macro::ChoiceCriteria;

#[derive(ChoiceCriteria)]
enum Category {
    /// Billing question or dispute.
    Billing,
    Technical,
    Other,
}

let criteria = Category::choice_criteria();
// [("billing", Some("Billing question or dispute.")), ("technical", None), ("other", None)]
```

Labels follow `#[serde(rename)]` / a subset of `#[serde(rename_all)]` (lowercase,
UPPERCASE, snake_case, SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE, camelCase,
PascalCase) when present, so the same enum can also derive `serde::Deserialize` for the
response side without the two disagreeing on wire labels. Unannotated, a label is the
variant name lowercased.

For the same result via runtime reflection instead of a proc-macro, see
[`systemone-facet`](../systemone-facet) — both are supported long-term; pick whichever
fits (self-contained macro vs. reusing an existing `facet` dependency).

See [`examples/typed.rs`](examples/typed.rs) for a full round trip against the live API
(`TYPESAFE_API_KEY=... cargo run --example typed -p systemone-macro`).

## License

MIT, see [`../LICENSE`](../LICENSE).
