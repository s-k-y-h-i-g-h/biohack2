# Build the biohack2 web frontend into a complete, deployable dist/.
#
# Produces:
#   dist/index.html          — loader shell
#   dist/biohack2_web.js     — wasm-bindgen JS glue
#   dist/biohack2_web_bg.wasm — optimized WASM (wasm-opt if available)
#   dist/styles/global.css   — styles
#   dist/manifest.json, sw.js, icon-*.png — PWA assets from web/public
#
# Usage: bash build-web.sh [--serve]
#   --serve  after building, serve dist/ at http://localhost:8082 (server.py)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> Building WASM (wasm-pack, release, target web)..."
wasm-pack build web --release --target web --out-dir ../dist --no-typescript \
  || wasm-pack build web --release --target web --no-typescript

# wasm-pack --out-dir ../dist writes into web/../dist = root dist/
DIST="$ROOT/dist"
[ -d "$DIST" ] || { echo "dist/ not found after wasm-pack" >&2; exit 1; }

echo "==> Optimizing WASM with wasm-opt (if available)..."
if command -v wasm-opt >/dev/null 2>&1; then
  if wasm-opt -Oz --strip-debug "$DIST/biohack2_web_bg.wasm" -o "$DIST/biohack2_web_bg.wasm.opt"; then
    mv "$DIST/biohack2_web_bg.wasm.opt" "$DIST/biohack2_web_bg.wasm"
    echo "    wasm-opt applied"
  else
    echo "    WARNING: wasm-opt failed — keeping unoptimized WASM" >&2
  fi
else
  echo "    wasm-opt not found — skipping (npm install -g binaryen)"
fi

echo "==> Copying app shell and PWA assets..."
mkdir -p "$DIST/styles"
cp web/index.html "$DIST/index.html"
cp web/src/styles/global.css "$DIST/styles/global.css"
cp web/public/manifest.json web/public/sw.js \
   web/public/icon-192.png web/public/icon-512.png "$DIST/"

echo "==> Done. Deployable output in dist/:"
ls -la "$DIST" | grep -vE "^total|^\.$|^\.\.$"
WASM_SIZE=$(stat -c %s "$DIST/biohack2_web_bg.wasm")
echo "WASM size: $((WASM_SIZE / 1024)) KB"
