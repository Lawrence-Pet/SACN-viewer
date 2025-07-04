#!/bin/bash

# sACN Viewer Launch Script
echo "Starting sACN Desktop Viewer..."
echo "This will listen for sACN packets on port 5568"
echo "Press Ctrl+C to stop"
echo ""

# Check if the release build exists
if [ -f "target/release/sacn-viewer" ]; then
    echo "Running release build..."
    ./target/release/sacn-viewer
else
    echo "Release build not found. Building and running debug version..."
    cargo run
fi
