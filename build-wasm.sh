#!/bin/bash
# Build script for JCL WebAssembly module

set -e

echo "🚀 Building JCL for WebAssembly..."

# Check if wasm32 target is installed
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 wasm-bindgen-cli not found. Installing..."
    cargo install wasm-bindgen-cli
fi

# Build the WASM module
echo "🔨 Building WASM module..."
cargo build --target wasm32-unknown-unknown --lib --no-default-features --features wasm --release

# Generate JavaScript bindings
echo "📝 Generating JavaScript bindings..."
wasm-bindgen target/wasm32-unknown-unknown/release/jcl.wasm \
    --out-dir pkg \
    --target web \
    --omit-default-module-path

# Get file sizes
WASM_SIZE=$(du -h pkg/jcl_bg.wasm | cut -f1)
echo "✅ Build complete!"
echo ""
echo "📊 Bundle size: $WASM_SIZE"
echo ""
echo "📂 Output files:"
echo "  - pkg/jcl.js          (JavaScript bindings)"
echo "  - pkg/jcl_bg.wasm     (WebAssembly binary)"
echo "  - pkg/jcl.d.ts        (TypeScript definitions)"
echo ""
echo "🌐 To test the playground:"
echo "  1. Start a local web server:"
echo "     python -m http.server 8000"
echo "     # or"
echo "     npx http-server"
echo ""
echo "  2. Open in browser:"
echo "     http://localhost:8000/playground.html"
echo ""
echo "🎉 Happy coding!"
