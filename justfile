default:
    @just --list

build *ARGS:
    cargo build --workspace {{ARGS}}

check *ARGS:
    cargo check --workspace --tests {{ARGS}}

test *ARGS:
    cargo test --workspace {{ARGS}}

lint:
    cargo clippy --workspace --all-targets -- -D warnings

lint-fix:
    cargo clippy --workspace --all-targets --fix

fmt-check:
    cargo fmt --all -- --check

fmt:
    cargo fmt --all

doc *ARGS:
    cargo doc --workspace --no-deps {{ARGS}}

ci: fmt-check lint test doc
