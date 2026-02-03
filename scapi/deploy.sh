#!/bin/bash
set -e # Exit immediately if a command exits with a non-zero status.

echo "Build and Deploy Script for scapi"

# 1. Build the project
# This uses the Wasmer toolchain (cargo-wasix) to compile the project to WebAssembly.
# Ensure you have a working local environment or use the GitHub Action for cloud builds.
echo "Running 'wasmer build'..."
wasmer build

# 2. Deploy to Wasmer Edge
# This uploads the package (and the built scapi.wasm) to Wasmer's cloud.
echo "Running 'wasmer deploy'..."
wasmer deploy --non-interactive

echo "Done!"
