#!/bin/bash
set -e

echo "Building nori-lint..."
echo ""

echo "[1/3] Compiling TypeScript..."
tsc
echo "Done"
echo ""

echo "[2/3] Resolving path aliases..."
tsc-alias
echo "Done"
echo ""

echo "[3/3] Setting file permissions..."
chmod +x build/src/cli.js
echo "Done"
echo ""

echo "Build complete"
echo "Build output: build/"
