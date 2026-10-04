#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
cd "$SCRIPT_DIR"

set -a
source "$SCRIPT_DIR/.env"
set +a

flutter build macos \
  "--dart-define=RDP_ENDPOINT=$RDP_ENDPOINT" \
  "--dart-define=RDP_TOKEN=$RDP_TOKEN"
