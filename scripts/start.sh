#!/usr/bin/env bash

set -euo pipefail

# -----------------------------------------------------------------------------
# AskRepo Docker start helper
# -----------------------------------------------------------------------------

usage() {
  cat <<'EOF'
Usage: scripts/start.sh [options]

Run the AskRepo Docker image. The script ensures the image exists locally and
launches the container using the project .env file.

Options:
  -t, --tag <value>   Image tag to run (defaults to Cargo version or 'latest')
  -n, --name <value>  Container name (defaults to askrepo)
  -d, --detach        Run container in detached mode
  -h, --help          Show this help message
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_BASENAME="askrepo-service"
TAG=""
ENV_FILE="$PROJECT_ROOT/.env"
CONTAINER_NAME="askrepo"
DETACH=false

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
    -d|--detach)
      DETACH=true
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

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Environment file not found: $ENV_FILE" >&2
  exit 1
fi

# Ensure the image is available locally; build if missing.
if ! docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
  echo "[askrepo] Image '${IMAGE_NAME}' not found locally; building..."
  "$SCRIPT_DIR/build.sh" --tag "$TAG"
fi

RUN_ARGS=(
  --rm
  --name "$CONTAINER_NAME"
  --env-file "$ENV_FILE"
)

if $DETACH; then
  RUN_ARGS+=(-d)
else
  RUN_ARGS+=(-it)
fi

echo "[askrepo] Starting container '${CONTAINER_NAME}' from image '${IMAGE_NAME}'"
docker run "${RUN_ARGS[@]}" "$IMAGE_NAME"
