#!/usr/bin/env bash
set -euo pipefail

echo "Running pre-push checks..."

echo "--- cargo fmt ---"
cargo fmt --check

echo "--- cargo check ---"
cargo check

echo "--- cargo clippy ---"
cargo clippy -- -D warnings

echo "--- cargo test ---"
cargo test

echo "All pre-push checks passed!"
