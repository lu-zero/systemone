default:
    @just --list

build *ARGS:
    cargo build --workspace {{ARGS}}

check *ARGS:
    cargo check --workspace --tests {{ARGS}}

test *ARGS:
    cargo test --workspace {{ARGS}}

lint:
    cargo clippy --all --tests -- -D warnings

lint-fix:
    cargo clippy --all --tests --fix

fmt-check:
    cargo fmt --all -- --check

fmt:
    cargo fmt --all

ci: fmt-check lint test
