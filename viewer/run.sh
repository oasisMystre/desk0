#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
cd "$SCRIPT_DIR"

set -a
source "$SCRIPT_DIR/.env"
set +a

RUST_LOG=debug flutter run -d macos \
  "--dart-define=CONSOLE_URL=$CONSOLE_URL" \
  "--dart-define=RDP_ENDPOINT=$RDP_ENDPOINT" \
  "--dart-define=RDP_TOKEN=$RDP_TOKEN" \
  "--dart-define=API_BASE_URL=$API_BASE_URL"
