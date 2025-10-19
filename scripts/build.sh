#!/usr/bin/env bash

set -euo pipefail

# -----------------------------------------------------------------------------
# AskRepo Docker build helper
# -----------------------------------------------------------------------------

usage() {
  cat <<'EOF'
Usage: scripts/build.sh [options]

Build the AskRepo Docker image.

Options:
  -t, --tag <value>     Tag to apply to the image (defaults to Cargo version or 'latest')
  -n, --no-cache        Build the image without using Docker layer cache
  -h, --help            Show this help message
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_BASENAME="askrepo-service"
TAG=""
NO_CACHE=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--tag)
      TAG="$2"
      shift 2
      ;;
    -n|--no-cache)
      NO_CACHE=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$TAG" ]]; then
  if [[ -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    TAG="$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -n 1 | cut -d '"' -f2)"
  fi
  TAG="${TAG:-latest}"
fi

IMAGE_NAME="${IMAGE_BASENAME}:${TAG}"

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required but not installed or not on PATH." >&2
  exit 1
fi

BUILD_ARGS=()
if $NO_CACHE; then
  BUILD_ARGS+=(--no-cache)
fi

echo "[askrepo] Building image '${IMAGE_NAME}'"
(
  cd "$PROJECT_ROOT"
  docker build "${BUILD_ARGS[@]}" -t "$IMAGE_NAME" .
)

echo "[askrepo] Build complete"
