# systemone-bench

Not published — internal criterion benchmark comparing
[`systemone-macro`](../systemone-macro)'s compile-time codegen against
[`systemone-facet`](../systemone-facet)'s runtime reflection for building the same
`choice` criteria from equivalent enums.

```sh
cargo bench -p systemone-bench
```

Latest measured result (steady state — `systemone-facet`'s lowercase-label cache is
filled once per process and that cost isn't visible here):

| Backend | Time per call |
|---------|---------------|
| `systemone-macro` | ~45ns |
| `systemone-facet` | ~237ns |

Both are negligible next to an actual HTTP round trip; this is a
compile-time-complexity/ecosystem-fit tradeoff, not a reason to prefer one at runtime.
