#!/usr/bin/env bash
# Render every example in this directory so you can verify what meraid supports.
#
# Usage:
#   examples/run-all.sh              # render all examples (text)
#   examples/run-all.sh --ascii      # pass extra flags through to meraid
#   examples/run-all.sh --theme neon
set -euo pipefail

# Prefer an installed `meraid`, fall back to a locally built release binary.
if command -v meraid >/dev/null 2>&1; then
  MERAID=meraid
elif [ -x "target/release/meraid" ]; then
  MERAID="./target/release/meraid"
elif [ -x "../target/release/meraid" ]; then
  MERAID="../target/release/meraid"
else
  echo "meraid not found. Build it first: cargo build --release" >&2
  exit 1
fi

DIR="$(cd "$(dirname "$0")" && pwd)"

for f in "$DIR"/*.mmd; do
  name="$(basename "$f")"
  printf '\n\033[1m━━━ %s ━━━\033[0m\n' "$name"
  printf '\033[2m$ meraid %s %s\033[0m\n\n' "examples/$name" "$*"
  "$MERAID" "$f" "$@"
done
