#!/usr/bin/env bash
set -euo pipefail

step() {
  printf '\n==> %s\n' "$1"
}

step "cargo fmt --all -- --check"
cargo fmt --all -- --check

step "cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

step "cargo test --workspace"
cargo test --workspace

step "cargo build --workspace --all-features"
cargo build --workspace --all-features

step "cargo doc --workspace --no-deps --all-features"
cargo doc --workspace --no-deps --all-features

step "validate gtpack wizard manifests"
bash ci/gtpacks.sh validate

step "done"
