#!/bin/sh
# Runs the project's checks locally (GitHub Actions is not available).
#
#   scripts/check.sh          formatting, clippy, tests and the wasm32 (web) build check
#   scripts/check.sh --audit  the same, plus `cargo audit` (downloads the advisory list)
#
# One-time setup:
#   rustup target add wasm32-unknown-unknown
#   cargo install cargo-audit
#   git config core.hooksPath .githooks   # runs this script before each push
set -eu

cd "$(dirname "$0")/.."

step() {
    printf '\n==> %s\n' "$*"
    "$@"
}

step cargo fmt --check
step cargo clippy --all-targets -- -D warnings
step cargo test
step cargo clippy --target wasm32-unknown-unknown --lib --bins -- -D warnings

if [ "${1:-}" = "--audit" ]; then
    step cargo audit
fi

printf '\nAll checks passed.\n'
