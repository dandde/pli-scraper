#!/bin/bash
set -e

# Build first to avoid build output in the run logs
echo "Building server..."
cargo build -p ferret_server

# Start server in background
echo "Starting server..."
PORT=3000 cargo run -p ferret_server &
SERVER_PID=$!

# Give it a moment to start
sleep 5

echo "Testing server with curl..."
RESPONSE=$(curl -s "http://127.0.0.1:3000/https://palimyanmarpitaka.blogspot.com/")

echo "Response received:"
echo "$RESPONSE"

# Check if response contains expected JSON keys
if echo "$RESPONSE" | grep -q "tags" && echo "$RESPONSE" | grep -q "html"; then
    echo "SUCCESS: Response looks like valid analysis JSON."
else
    echo "FAILURE: Response does not look correct."
    kill $SERVER_PID
    exit 1
fi

# Cleanup
kill $SERVER_PID
echo "Server stopped."
exit 0
