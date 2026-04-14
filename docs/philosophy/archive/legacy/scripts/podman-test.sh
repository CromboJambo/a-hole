#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="crabjar-dev"

podman run --rm -it \
    -v "${ROOT_DIR}:/workspace:rw,z" \
    -w /workspace \
    "${IMAGE_NAME}" \
    /bin/bash -lc "cargo test"
