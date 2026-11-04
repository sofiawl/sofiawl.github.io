#!/usr/bin/env bash
# Compile the Rust organism to WebAssembly and drop it next to the web page.
set -euo pipefail
cd "$(dirname "$0")"
rustup target add wasm32-unknown-unknown >/dev/null
command -v wasm-pack >/dev/null || cargo install wasm-pack
wasm-pack build crate --release --target web --out-dir ../web/pkg --no-typescript
rm -f web/pkg/.gitignore web/pkg/package.json web/pkg/README.md
cp web/index.html web/404.html  # GitHub Pages serves this for /work, /resume, ...
python3 tools/spores.py          # blog pages (no Rust needed for this step)
echo "Built. Serve the web/ folder, e.g.: python3 -m http.server 8080 -d web"
