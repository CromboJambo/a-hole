#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="crabjar-dev"

usage() {
    cat << 'EOF'
Usage: scripts/podman.sh <command>

Commands:
  build   Build the dev container image
  shell   Start a shell with the repo mounted at /workspace (rw)
  run     Run a one-off command in the container (pass after --)

Examples:
  scripts/podman.sh build
  scripts/podman.sh shell
  scripts/podman.sh run -- cargo build
EOF
}

cmd="${1:-}"
case "$cmd" in
    build)
        podman build -f "${ROOT_DIR}/Containerfile" -t "${IMAGE_NAME}" "${ROOT_DIR}"
        ;;
    shell)
        podman run --rm -it \
            -v "${ROOT_DIR}:/workspace:rw,z" \
            -w /workspace \
            "${IMAGE_NAME}" \
            /bin/bash
        ;;
    run)
        shift
        if [ "${1:-}" = "--" ]; then
            shift
        fi
        if [ "$#" -eq 0 ]; then
            echo "No command provided."
            usage
            exit 1
        fi
        podman run --rm -it \
            -v "${ROOT_DIR}:/workspace:rw,z" \
            -w /workspace \
            "${IMAGE_NAME}" \
            "$@"
        ;;
    *)
        usage
        exit 1
        ;;
esac
