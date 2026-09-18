# Project Conventions

`systemone` is a Rust client for the TypeSafe AI `systemone` API, built to eventually be
usable by [maki](https://github.com/tontinton/maki) — its dependency choices and code
style follow maki's conventions for that reason, not by default.

## Build commands

```bash
just check   # cargo check --workspace --tests
just test    # cargo test --workspace
just lint    # cargo clippy --workspace --all-targets -- -D warnings
just fmt     # cargo fmt --all
just doc     # cargo doc --workspace --no-deps
just ci      # all of the above, what CI runs
```

Match `.github/workflows/ci.yml` before opening a PR — `just ci` runs the same checks.
`cargo doc` warnings are hard errors in CI (`RUSTDOCFLAGS=-D warnings`).

## Architecture

- `systemone` — the client: HTTP transport (`isahc`), retry, errors, question builders,
  wire types.
- `systemone-macro` — `#[derive(ChoiceCriteria)]`, compile-time codegen from an enum.
- `systemone-facet` — the same via `facet` reflection at call time instead.
- `systemone-bench` — criterion benchmark comparing the two; not published.

`systemone-macro` and `systemone-facet` solve the same problem two ways on purpose —
neither supersedes the other. Don't consolidate them into one "winner" without being
asked.

## Code guidelines

- No trivial comments; minimal bloat (KISS/DRY/SRP); each line should justify its
  existence.
- No wildcard imports. No unnecessary state (variables, fields, arguments).
- Explicit `Result<T, E>` over panics; `thiserror` for domain errors, one flat enum per
  crate (not a class hierarchy) with helper methods on it (see `systemone::Error`).
- No inline magic numbers or strings.
- Unit tests live in the same file under `#[cfg(test)] mod tests`. Use `test-case` for
  parametrized cases; name tests `snake_case`, describing the behavior, not "test1".
- Executor-agnostic by design: `isahc` for HTTP, `async-io::Timer` for sleeps. No
  `tokio`/`smol` dependency in library code.

## Dependencies

- Add a dependency to the root `[workspace.dependencies]` and reference it with
  `{ workspace = true }` from the member — *except* a proc-macro crate's own
  `syn`/`quote`/`proc-macro2`, and a crate-specific dependency only one member needs
  (`facet`, `criterion`): those stay local to that member's `Cargo.toml`.
- Semver requirements (`"3"`), never exact pins (`"=3.0.6"`). `Cargo.lock` is gitignored.
- Try solving with an existing dependency before adding a new one.

## Unslop rules

Enforced, not a stylistic preference. This codebase is written and maintained with heavy
AI assistance, and every rule below is something a real pass got wrong and had to clean
up later (see the retry-loop/error-enum simplification in git history for what "before"
looked like).

**Comments**
- Default to no comment. Add one only when the *why* is non-obvious: a hidden
  constraint, a workaround for a specific bug, an invariant a future reader would
  violate without warning.
- Hard limits: no comment line over 150 characters, no comment block over 3 lines.
  Hitting either means cut, don't wrap — needing more room is a sign the explanation (or
  the code it's attached to) needs to be smaller, not that it needs more space.
- Never restate what the code already says through its own names — the reader can read
  Rust.
- Never reference the current task, a commit, a PR/issue number, or a session ("fixed
  for the X flow", "added per review", "per Luca's request"). That context belongs in
  the commit message; a comment that cites it rots the moment the file moves.
- No commented-out code and no `// removed: ...` markers for deleted code — `git
  log`/`git blame` is the actual history.

**Dead weight**
- No speculative abstraction for a single call site: no config knob, trait
  generalization, or feature flag without a second concrete caller that needs it today.
- No error handling, fallback, or validation for a scenario the caller's own guarantees
  already rule out.
- `#[allow(dead_code)]` is not a way to keep something "just in case" — delete it; it's
  in git history if it turns out to be needed.

**Tests**
- A test must exercise a real branch or decision point in *this* code, not the standard
  library or a dependency it thinly wraps. `assert!(Client::builder().build().is_err())`
  with no API key tests our validation; asserting `2 + 2 == 4` in the middle of it would
  not.
- No test added purely to make "touched a function → added a test" true.

Before adding either a comment or an abstraction, ask: would a maintainer who knows this
codebase well have bothered to write this, or does it exist because generating
*something* felt safer than generating nothing?

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), enforced:

```
<type>: <description>
```

- `feat:` — new functionality
- `fix:` — bug fix
- `refactor:` — code restructuring without behavior change
- `docs:` — documentation only (README, AGENTS.md, doc comments)
- `test:` — adding or updating tests
- `ci:` — CI/CD changes
- `chore:` — maintenance (dependencies, tooling, release housekeeping)

Subject line imperative mood ("add", not "added"/"adds"), no period, ideally under 72
characters. A scope (`fix(retry): ...`) is fine but optional. A body, if there is one,
follows the same hard limits as comments above: no line over 150 characters, no
paragraph over 5 lines (3 is better) — state what changed and why it matters to a future
reader, not a narrated investigation.

## MSRV

Workspace floor is 1.88 (matches maki). `systemone-facet` and `systemone-bench` need
1.90 for the `facet` crate and declare that in their own `rust-version`, overriding the
workspace default rather than raising it for everyone — most of this workspace has no
reason to need a newer toolchain than maki does.
