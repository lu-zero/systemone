# systemone-core

The `no_std` + `alloc` protocol half of the [`systemone`](https://crates.io/crates/systemone)
client: wire types, question builders, retry policy, and the OpenAI dialect encode/decode.
No HTTP, no sleep, no environment access. Hosts (the maki client, firmware) provide the
transport and the RNG.

License: MIT
