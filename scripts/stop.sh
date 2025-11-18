#!/usr/bin/env bash

set -euo pipefail

# -----------------------------------------------------------------------------
# AskRepo Docker stop helper
# -----------------------------------------------------------------------------

usage() {
  cat <<'EOF'
Usage: scripts/stop.sh [options]

Stop the AskRepo container (if running) and remove the local Docker image.

Options:
  -t, --tag <value>   Image tag to remove (defaults to Cargo version or 'latest')
  -n, --name <value>  Container name to stop (defaults to askrepo)
  -h, --help          Show this help message
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_BASENAME="askrepo-service"
TAG=""
CONTAINER_NAME="askrepo"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--tag)
      TAG="$2"
      shift 2
      ;;
    -n|--name)
      CONTAINER_NAME="$2"
      shift 2
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

CONTAINER_ID="$(docker ps -aq --filter "name=^${CONTAINER_NAME}$" | head -n 1 || true)"
if [[ -n "$CONTAINER_ID" ]]; then
  if [[ -n "$(docker ps -q --filter "id=${CONTAINER_ID}")" ]]; then
    echo "[askrepo] Stopping running container '${CONTAINER_NAME}'"
    docker stop "$CONTAINER_NAME" >/dev/null
  fi
  if docker ps -aq --filter "id=${CONTAINER_ID}" >/dev/null 2>&1; then
    echo "[askrepo] Removing container '${CONTAINER_NAME}'"
    docker rm "$CONTAINER_NAME" >/dev/null 2>&1 || true
  fi
fi

if docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
  echo "[askrepo] Removing image '${IMAGE_NAME}'"
  docker image rm "$IMAGE_NAME"
else
  echo "[askrepo] Image '${IMAGE_NAME}' not found locally; nothing to remove."
fi
