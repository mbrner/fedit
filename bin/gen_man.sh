#!/usr/bin/env bash
set -euo pipefail

# Generate a groff manpage for the fedit CLI using help2man.
# Usage: bin/gen_man.sh [CLI_BINARY] [OUTPUT_MANPAGE]
# Defaults: CLI_BINARY=bin/fedit.py, OUTPUT_MANPAGE=man/fedit.1
BIN="${1:-bin/fedit.py}"
OUT="${2:-man/fedit.1}"

if ! command -v help2man >/dev/null 2>&1; then
  echo "Error: help2man is required to generate a manpage." >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"
help2man -N \
  -n "FEdit: whitespace-insensitive search and replace with encoding and line ending preservation." \
  -o "$OUT" \
  --version-string "0.1.0" \
  "$BIN"
echo "Wrote manpage to $OUT"
