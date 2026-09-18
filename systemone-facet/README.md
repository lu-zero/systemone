# systemone-facet

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/systemone-facet.svg)](https://crates.io/crates/systemone-facet)
[![Docs.rs](https://docs.rs/systemone-facet/badge.svg)](https://docs.rs/systemone-facet)

Typed `choice` criteria for [`systemone`](../systemone) via
[`facet`](https://github.com/facet-rs/facet) reflection, as an alternative to
[`systemone-macro`](../systemone-macro)'s derive macro.

```rust
use facet::Facet;
use systemone::ChoiceCriteria;

#[derive(Facet)]
#[repr(u8)]
enum Category {
    /// Billing question or dispute.
    Billing,
    Technical,
}
systemone_facet::impl_choice_criteria!(Category);

let criteria = Category::choice_criteria();
```

`impl_choice_criteria!` exists because a blanket `impl<T: Facet> ChoiceCriteria for T`
is barred by Rust's orphan rule (both the trait and `Facet` are foreign to this crate) —
so each type gets this one-line bridge instead. Labels come from `#[facet(rename)]` /
`#[facet(rename_all)]`, else the variant name lowercased (matching `systemone-macro`'s
default); descriptions come from doc comments. The lowercased fallback is cached
globally after first use, since reaching `'static` requires leaking it once.

Requires Rust 1.90+ (`facet`'s MSRV), above the rest of the workspace's 1.88 floor.

See [`examples/typed_facet.rs`](examples/typed_facet.rs) for a full round trip against
the live API (`TYPESAFE_API_KEY=... cargo run --example typed_facet -p systemone-facet`).

## License

MIT, see [`../LICENSE`](../LICENSE).
