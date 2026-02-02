#!/bin/bash
set -e

echo "Building scapi..."
cargo build -p scapi

echo "Starting scapi on port 3000..."
export PORT=3000
# Run the binary directly or via cargo run (cargo run is convenient for dev)
cargo run -p scapi
