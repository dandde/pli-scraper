#!/bin/bash
set -e

# Build first to avoid build output in the run logs
echo "Building server..."
cargo build -p scapi

# Start server in background
echo "Starting server..."
# Kill any existing server on port 3000
lsof -ti:3000 | xargs kill -9 2>/dev/null

export PORT=3000 
cargo run -p scapi &
SERVER_PID=$!

# Give it a moment to start
sleep 5

# echo "Testing server with curl..."
# RESPONSE=$(curl -s "http://127.0.0.1:3000/api/report/https://palimyanmarpitaka.blogspot.com/")

# echo "Response received:"
# echo "$RESPONSE"

# Check if response contains expected JSON keys
# if echo "$RESPONSE" | grep -q "tags" && echo "$RESPONSE" | grep -q "html"; then
#     echo "SUCCESS: Response looks like valid analysis JSON."
# else
#     echo "FAILURE: Response does not look correct."
#     kill $SERVER_PID
#     exit 1
# fi

echo "Testing server with format=tree..."
# We just check if it returns 200 OK and valid JSON, assuming stdout is handled by server
RESPONSE_TREE=$(curl -s "http://127.0.0.1:3000/api/report/https://palimyanmarpitaka.blogspot.com/?format=tree")

if echo "$RESPONSE_TREE" | grep -q "Files analyzed"; then
    echo "SUCCESS: Tree report request returned valid text."
else
    echo "FAILURE: Tree report request failed."
    kill $SERVER_PID
    exit 1
fi

echo "Testing server with export format=csv..."
RESPONSE_CSV=$(curl -s "http://127.0.0.1:3000/api/export/https://palimyanmarpitaka.blogspot.com/?format=csv")

if echo "$RESPONSE_CSV" | grep -q "Tag,Count,Attribute"; then
    echo "SUCCESS: CSV export request returned valid CSV content."
else
    echo "FAILURE: CSV export request failed."
    echo "Response was: $RESPONSE_CSV"
    kill $SERVER_PID
    exit 1
fi

# Cleanup
kill $SERVER_PID
echo "Server stopped."
exit 0
