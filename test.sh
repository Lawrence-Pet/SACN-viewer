#!/bin/bash

# Test sACN Sender Script using the sacn crate
echo "sACN Viewer Test Script"
echo "This script demonstrates the updated sACN implementation using the official sacn crate"
echo ""

# Build the main project first
echo "Building sACN viewer with sacn crate..."
cargo build

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
    echo ""
    echo "You can now:"
    echo "1. Run the sACN viewer GUI: ./target/debug/sacn-viewer"
    echo "2. Use the built-in test sender (see test_sender.rs for example code)"
    echo ""
    echo "The application now uses the official sacn crate which provides:"
    echo "- Standards-compliant sACN implementation (ANSI E1.31-2018)"
    echo "- Automatic universe discovery"
    echo "- Source discovery and tracking"
    echo "- Synchronization support"
    echo "- Robust error handling"
    echo "- Cross-platform compatibility"
    echo ""
    echo "To test sending, you can modify and run the test_sender.rs with:"
    echo "cargo run --bin test_sender"
else
    echo "✗ Build failed"
    exit 1
fi
