#!/usr/bin/env bash
# Build the complete biohack2 stack: WASM frontend + server binary.
#
# Produces:
#   dist/                  — static frontend (index.html, WASM, JS, PWA assets)
#   target/release/biohack2-server  — axum + SQLite backend
#
# Usage: bash build-all.sh [--serve]
#   --serve  after building, start the server on http://localhost:8082

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> Building server binary (release)..."
cargo build --release --bin biohack2-server

echo "==> Building WASM frontend..."
bash build-web.sh "$@"

echo "==> Done. Run with:"
echo "    ./target/release/biohack2-server"
echo "  or"
echo "    npm run serve"