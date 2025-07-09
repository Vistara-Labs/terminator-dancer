#!/bin/bash

# 🚀 Terminator-Dancer WASM Build Script
# Compiles the Rust runtime to WebAssembly for browser demo

set -e

echo "🤖 TERMINATOR-DANCER WASM BUILD"
echo "==============================="

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check if basic-http-server is installed
if ! command -v basic-http-server &> /dev/null; then
    echo "📦 Installing basic-http-server..."
    cargo install basic-http-server
fi

echo "🔧 Building WASM module..."

# Build for web target with optimizations
wasm-pack build \
    --target web \
    --out-dir web_demo/pkg \
    --features wasm

echo "✅ WASM build complete!"

# Copy HTML demo file
echo "📋 Setting up web demo..."

# Create directory structure
mkdir -p web_demo/pkg

# Optionally start the server
read -p "🌐 Start demo server now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 Starting demo server..."
    cd web_demo
    echo "🌐 Demo running at: http://localhost:4000"
    echo "💻 Press Ctrl+C to stop"
    basic-http-server .
fi 