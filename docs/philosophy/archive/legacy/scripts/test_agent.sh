#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Running Crabjar checks from ${ROOT_DIR}"
cd "${ROOT_DIR}"

echo "1) Workspace check"
cargo check --workspace

echo "2) Root crate tests"
cargo test -p crabjar

echo "All checks passed."
