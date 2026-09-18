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

This codebase is written and maintained with heavy AI assistance — treat these as
enforced, not stylistic preference:

- Default to no comment. Add one only when the *why* is non-obvious: a hidden
  constraint, a workaround, an invariant a future reader would otherwise violate.
- Terse: one sentence beats a paragraph. No comment block over ~3 lines; if it needs
  more, the code needs simplifying instead, or the explanation belongs in the commit
  message.
- Never restate what the code already says through its own names.
- Never reference the current task, a commit, a PR/issue number, or a session ("fixed
  for the X flow", "added per review") — that belongs in the commit message, not code
  that outlives it.
- No commented-out code, no `// removed: ...` markers — `git log`/`git blame` is the
  actual history.

## Dead weight

- No speculative abstraction for a single call site: no config knob, trait
  generalization, or feature flag without a second concrete caller that needs it today.
- No error handling or validation for a scenario the caller's own guarantees already
  rule out.
- `#[allow(dead_code)]` is not a way to keep something "just in case" — delete it; it's
  in git history if it turns out to be needed.

## Commits

Reasonably close to [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `ci:`, `chore:`) for the subject line;
not strictly enforced for a project this size, but prefer it.

## MSRV

Workspace floor is 1.88 (matches maki). `systemone-facet` and `systemone-bench` need
1.90 for the `facet` crate and declare that in their own `rust-version`, overriding the
workspace default rather than raising it for everyone — most of this workspace has no
reason to need a newer toolchain than maki does.
