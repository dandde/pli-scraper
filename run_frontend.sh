#!/bin/bash
set -e

cd frontend

if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

echo "Starting frontend..."
echo "This will open your default browser."
npm run dev -- --open
